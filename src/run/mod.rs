#[cfg(feature = "racer-completion")]
use crate::complete::code::{CodeCache, CodeCompleter};
use crate::complete::{cmdr::TreeCompleter, modules::ModulesCompleter};
use crate::prelude::*;
use crossterm::{event::Event, ExecutableCommand};
use kserd::{fmt::FormattingConfig, Kserd};
use repl::{EvalResult, Evaluate, Read, ReadResult};
use std::io::{self, prelude::*};
use std::sync::{Arc, Mutex};

mod interface;

use interface::{CItem, InputBuffer, Screen};

const CODE_COMPLETIONS: Option<usize> = Some(10);

#[cfg(feature = "racer-completion")]
struct CacheWrapper(CodeCache);
#[cfg(not(feature = "racer-completion"))]
struct CacheWrapper;

/// Get the terminal width, if possible.
pub fn terminal_width() -> Option<usize> {
    crossterm::terminal::size().map(|x| x.0 as usize).ok()
}

/// Creates a `FormattingConfig` that has a width based on the terminal width.
///
/// If the terminal width is less than 120, the size limit width is `terminal-width - 2`. If the
/// terminal width is greater than 120, the size limit is `max(80% * terminal-width, 120)`.
///
/// This is the function used by the REPL main entry point.
pub fn fmt_based_on_terminal_width() -> FormattingConfig {
    terminal_width()
        .map(|width| {
            let mut fmt = FormattingConfig::default();
            if width <= 120 {
                fmt.width_limit = Some(width.saturating_sub(2));
            } else {
                fmt.width_limit = Some(std::cmp::max((width * 4) / 5, 120));
            }
            fmt
        })
        .unwrap_or_default()
}

/// A container for callbacks when running the REPL.
///
/// These callbacks are mostly around setting up formatting and feeding evaluation results to the
/// caller.
pub struct RunCallbacks<'a, D, T, U> {
    evalfn: Box<dyn FnMut(Repl<Evaluate, D>) -> EvalResult<D> + 'a>,
    fmtrfn: Option<T>,
    resultfn: Option<U>,
}

impl<'a, D>
    RunCallbacks<'a, D, fn() -> FormattingConfig, fn(usize, Kserd<'static>, &Repl<Read, D>)>
{
    /// New callback using a synchronous model of data ownership. (eg `eval`).
    pub fn new(app_data: &'a mut D) -> Self {
        Self {
            evalfn: Box::new(move |repl| repl.eval(app_data)),
            fmtrfn: None,
            resultfn: None,
        }
    }

    /// New callback using asynchronous model of data ownership. (eg `eval_async`).
    pub fn new_async(app_data: Arc<Mutex<D>>) -> Self
    where
        D: Send + 'static,
    {
        Self {
            evalfn: Box::new(move |repl| repl.eval_async(&app_data).wait()),
            fmtrfn: None,
            resultfn: None,
        }
    }
}

impl<'a, D, T, U> RunCallbacks<'a, D, T, U> {
    /// Specify code to be run which dictates the formatting configuration to use.
    pub fn with_fmtrfn<F>(self, f: F) -> RunCallbacks<'a, D, F, U>
    where
        F: FnMut() -> FormattingConfig,
    {
        let RunCallbacks {
            evalfn, resultfn, ..
        } = self;
        RunCallbacks {
            evalfn,
            fmtrfn: Some(f),
            resultfn,
        }
    }

    /// Specify code to be run after evaluation has succeeded and a `Kserd` result is returned.
    ///
    /// The closure supplies the statement index `usize` and the result `Kserd`, along with the
    /// `Repl`.
    pub fn with_resultfn<F>(self, f: F) -> RunCallbacks<'a, D, T, F>
    where
        F: FnMut(usize, Kserd<'static>, &Repl<Read, D>),
    {
        let RunCallbacks { evalfn, fmtrfn, .. } = self;
        RunCallbacks {
            evalfn,
            fmtrfn,
            resultfn: Some(f),
        }
    }
}

/// Available with the `runnable` feature and when the REPL is in the `Read` state.
impl<D> Repl<Read, D> {
    /// Run the repl inside the terminal, consuming the repl. Returns the output of the REPL.
    pub fn run<T, U>(self, run_callbacks: RunCallbacks<D, T, U>) -> io::Result<String>
    where
        T: FnMut() -> kserd::fmt::FormattingConfig,
        U: FnMut(usize, kserd::Kserd<'static>, &Repl<Read, D>),
    {
        run(self, run_callbacks)
    }
}

fn run<D, FmtrFn, ResultFn>(
    mut read: Repl<Read, D>,
    mut runcb: RunCallbacks<D, FmtrFn, ResultFn>,
) -> io::Result<String>
where
    FmtrFn: FnMut() -> kserd::fmt::FormattingConfig,
    ResultFn: FnMut(usize, kserd::Kserd<'static>, &Repl<Read, D>),
{
    // set a custom panic handler to dump to a file
    // must be done as the screen captures the io streams and will
    // lose the panic message
    // if removed ensure `take_hook` is removed as well.
    let app_name = read.data.cmdtree.root_name().to_owned();
    std::panic::set_hook(Box::new(move |info| {
        let backtrace = backtrace::Backtrace::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let filename = format!(
            "{}.{}-{:03}.crash-report",
            app_name,
            now.as_secs(),
            now.subsec_millis()
        );
        let content =
            construct_crash_report(Vec::new(), &app_name, info, backtrace).unwrap_or_default();
        std::fs::write(filename, content).ok();
    }));

    let mut screen = interface::Screen::new()?;

    #[cfg(feature = "racer-completion")]
    let cache = {
        let cache = match CodeCache::new() {
            Ok(c) => c,
            Err((c, msg)) => {
                println!("warning: could not find rust src code: {}", msg);
                c
            }
        };
        CacheWrapper(cache)
    };
    #[cfg(not(feature = "racer-completion"))]
    let cache = CacheWrapper;

    let mut reevaluate: Option<String> = None;

    let output = loop {
        io::stdout()
            .execute(crossterm::style::Print(read.prompt()))
            .ok();

        let mut input_buf = interface::InputBuffer::new();

        if let Some(buf) = read.data.editing_src.take() {
            if reevaluate.is_none() {
                input_buf.insert_str(&buf);
            }
        }

        if let Some(val) = reevaluate.take() {
            read.line_input(&val);
        } else {
            if do_read(&mut read, &mut screen, input_buf, &cache)? {
                break read.output().to_owned();
            }
        }

        match read.read() {
            ReadResult::Read(repl) => read = repl,
            ReadResult::Eval(repl) => {
                match do_eval(repl, &mut runcb) {
                    Ok((repl, reeval)) => {
                        read = repl;
                        reevaluate = reeval;

                        // prep for next read
                        interface::erase_current_line(io::stdout())?.flush()?;
                    }
                    Err(r) => break r.output().to_owned(),
                }
            }
        }
    };

    let _ = std::panic::take_hook(); // remove the previous set_hook
    Ok(output)
}

fn construct_crash_report(
    mut content_buf: Vec<u8>,
    app_name: &str,
    info: &std::panic::PanicInfo,
    backtrace: backtrace::Backtrace,
) -> io::Result<Vec<u8>> {
    let content = &mut content_buf;

    writeln!(
        content,
        "An unhandled error occurred in the operation of {}.",
        app_name
    )?;
    writeln!(
        content,
        "Please send this information to the required parties."
    )?;

    writeln!(content, "\nPanic Payload:")?;
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        writeln!(content, "{}", s)?;
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        writeln!(content, "{}", s)?;
    } else {
        writeln!(content, "Unknown panic")?;
    }

    writeln!(content, "\nLocation:")?;
    if let Some(l) = info.location() {
        writeln!(content, "{}", l)?;
    } else {
        writeln!(content, "no location information")?;
    }

    writeln!(content, "\nBacktrace:\n{:?}", backtrace)?;

    Ok(content_buf)
}

/// Returns true if interrupt occurred.
fn do_read<D>(
    repl: &mut Repl<Read, D>,
    screen: &mut Screen,
    buf: InputBuffer,
    cache: &CacheWrapper,
) -> io::Result<bool> {
    use crossterm::event::{Event::*, KeyCode::*, KeyEvent, KeyModifiers};
    const ENTER: Event = Key(KeyEvent {
        modifiers: KeyModifiers::empty(),
        code: Enter,
    });
    const TAB: Event = Key(KeyEvent {
        modifiers: KeyModifiers::empty(),
        code: Tab,
    });
    const BREAK: Event = Key(KeyEvent {
        modifiers: KeyModifiers::CONTROL,
        code: Char('c'),
    });
    const STOPEVENTS: &[Event] = &[ENTER, TAB, BREAK];

    crossterm::terminal::enable_raw_mode().map_err(|e| map_xterm_err(e, "enabling raw mode"))?;

    let mut i = Some(buf);

    let initial = crossterm::cursor::position().unwrap_or((0, 0));

    let mut completion_writer = interface::CompletionWriter::new();

    let rdata = &repl.data;
    let treecmpltr = TreeCompleter::build(&rdata.cmdtree);
    let modscmpltr = ModulesCompleter::build(&rdata.cmdtree, rdata.mods_map());
    #[cfg(feature = "racer-completion")]
    let codecmpltr = CodeCompleter::build(rdata);

    loop {
        let (mut input, ev) = interface::read_until(screen, initial, i.take().unwrap(), STOPEVENTS);

        if ev == ENTER {
            repl.line_input(&input.buffer());
            write!(&mut io::stdout(), "\n\r")?;
            crossterm::terminal::disable_raw_mode()
                .map_err(|e| map_xterm_err(e, "disabling raw mode"))?;
            break Ok(false);
        } else if ev == TAB {
            let line = input.buffer();
            if completion_writer.is_same_input(&line) {
                completion_writer.next_completion();
            } else {
                let f = |start| input.ch_len().saturating_sub(line[start..].chars().count());

                let tree_chpos = f(TreeCompleter::word_break(&line));
                let mods_chpos = f(ModulesCompleter::word_break(&line));
                #[cfg(feature = "racer-completion")]
                let code_chpos = f(CodeCompleter::word_break(&line));

                let completions = if line.starts_with(crate::CMD_PREFIX) {
                    Box::new(std::iter::empty()) as Box<dyn Iterator<Item = CItem>>
                } else {
                    #[cfg(feature = "racer-completion")]
                    let c = {
                        let injection = format!("{}\n{}", repl.input_buffer(), line);
                        complete_code(&codecmpltr, &cache.0, &injection, code_chpos)
                    };

                    #[cfg(not(feature = "racer-completion"))]
                    let c = std::iter::empty();

                    Box::new(c) as Box<dyn Iterator<Item = CItem>>
                };

                let completions = completions
                    .chain(complete_cmdtree(&treecmpltr, &line, tree_chpos))
                    .chain(complete_mods(&modscmpltr, &line, mods_chpos));

                completion_writer.new_completions(completions);
            }

            completion_writer.overwrite_completion(initial, &mut input)?;
        } else if ev == BREAK {
            crossterm::terminal::disable_raw_mode()
                .map_err(|e| map_xterm_err(e, "disabling raw mode"))?;
            break Ok(true);
        }

        i = Some(input); // prep for next loop
    }
}

fn complete_cmdtree<'a>(
    tree: &'a TreeCompleter,
    line: &'a str,
    chpos: usize,
) -> impl Iterator<Item = CItem> + 'a {
    tree.complete(line).map(move |x| CItem {
        matchstr: x.0.to_owned(),
        input_chpos: chpos,
    })
}

fn complete_mods<'a>(
    mods: &'a ModulesCompleter,
    line: &'a str,
    chpos: usize,
) -> impl Iterator<Item = CItem> + 'a {
    mods.complete(line).map(move |x| CItem {
        matchstr: x,
        input_chpos: chpos,
    })
}

#[cfg(feature = "racer-completion")]
fn complete_code(
    code: &CodeCompleter,
    cache: &CodeCache,
    injection: &str,
    chpos: usize,
) -> impl Iterator<Item = CItem> {
    code.complete(injection, CODE_COMPLETIONS, cache)
        .into_iter()
        .map(move |x| CItem {
            matchstr: x.matchstr,
            input_chpos: chpos,
        })
}

fn do_eval<D, FmtrFn, ResultFn>(
    mut repl: Repl<Evaluate, D>,
    runcb: &mut RunCallbacks<D, FmtrFn, ResultFn>,
) -> Result<(Repl<Read, D>, Option<String>), Repl<Read, D>>
where
    FmtrFn: FnMut() -> kserd::fmt::FormattingConfig,
    ResultFn: FnMut(usize, kserd::Kserd<'static>, &Repl<Read, D>),
{
    let rx = repl.output_listen();

    let jh = std::thread::spawn(move || {
        let mut covered_lines = 0;
        for chg in rx.iter() {
            covered_lines = interface::write_output_chg(covered_lines, chg).unwrap_or(0);
        }
    });

    let r = (runcb.evalfn)(repl);

    // prepare the formatter for output
    let fmt = runcb.fmtrfn.as_mut().map(|f| f()).unwrap_or_default();

    let (mut read, signal) = {
        let (repl, signal) = (r.repl, r.signal);
        let (repl, result) = repl.print_with_formatting(fmt);
        if let Some((idx, kserd)) = result {
            if let Some(f) = &mut runcb.resultfn {
                f(idx, kserd, &repl);
            }
        }
        (repl, signal)
    };

    read.close_channel();
    jh.join().unwrap();

    match signal {
        Signal::None => Ok((read, None)),
        Signal::Exit => Err(read),
        Signal::ReEvaluate(val) => Ok((read, Some(val))),
    }
}

fn map_xterm_err(xtermerr: crossterm::ErrorKind, msg: &str) -> io::Error {
    match xtermerr {
        crossterm::ErrorKind::IoError(e) => e,
        _ => io::Error::new(io::ErrorKind::Other, msg),
    }
}
