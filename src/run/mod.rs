#[cfg(feature = "racer-completion")]
use crate::complete::code::{CodeCache, CodeCompleter};
use crate::complete::{cmdr::TreeCompleter, modules::ModulesCompleter};
use crate::prelude::*;
use crossterm as xterm;
use crossterm::event::Event;
use kserd::{fmt::FormattingConfig, Kserd};
use repl::{EvalResult, Evaluate, Print, Read, ReadResult};
use std::io::{self, prelude::*};
use std::sync::{Arc, Mutex};

mod interface;
#[cfg(test)]
mod tests;

use interface::{CItem, Interface, Screen};

#[cfg(feature = "racer-completion")]
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
/// If the terminal width is less than 120, the size limit width is `terminal-width - prompt - 2`. If the
/// terminal width is greater than 120, the size limit is `max(80% * terminal-width, 120) - prompt`.
///
/// This is the function used by the REPL main entry point.
pub fn fmt_based_on_terminal_width<D>(repl: &Repl<Print, D>) -> FormattingConfig {
    terminal_width()
        .map(|width| {
            let mut fmt = FormattingConfig::default();
            let width = if width <= 120 {
                width.saturating_sub(2)
            } else {
                std::cmp::max((width * 4) / 5, 120)
            }
            .saturating_sub(repl.prompt(false).chars().count());
            fmt.width_limit = Some(width);
            fmt
        })
        .unwrap_or_default()
}

/// Used to capture the `D` reference within the RunCallbacks. Both variants of `D` are stored, and
/// an associated evaluation function is used. The closure is stored so that the run functions
/// don't require D: Send + 'static, instead this bound is only required on the creation of a
/// RunCallbacks which is using the _async_ variant.
#[allow(clippy::type_complexity)]
enum Data<'a, D> {
    Sync(
        &'a mut D,
        Box<dyn FnMut(Repl<Evaluate, D>, &mut D) -> EvalResult<D>>,
    ),
    Async(
        Arc<Mutex<D>>,
        Box<dyn FnMut(Repl<Evaluate, D>, &Arc<Mutex<D>>) -> EvalResult<D>>,
    ),
}

/// A container for callbacks when running the REPL.
///
/// These callbacks are mostly around setting up formatting and feeding evaluation results to the
/// caller.
pub struct RunCallbacks<'a, D, T, U, V> {
    data: Data<'a, D>,
    fmtrfn: Option<T>,
    resultfn: Option<U>,
    exitfn: Option<V>,
}

impl<'a, D>
    RunCallbacks<
        'a,
        D,
        fn(&Repl<Print, D>) -> FormattingConfig,
        fn(usize, Kserd<'static>, &Repl<Read, D>),
        fn(&mut ReplData<D>, &mut D),
    >
{
    /// New callback using a synchronous model of data ownership. (eg `eval`).
    pub fn new(app_data: &'a mut D) -> Self {
        Self {
            data: Data::Sync(app_data, Box::new(|repl, data| repl.eval(data))),
            fmtrfn: None,
            resultfn: None,
            exitfn: None,
        }
    }

    /// New callback using asynchronous model of data ownership. (eg `eval_async`).
    pub fn new_async(app_data: Arc<Mutex<D>>) -> Self
    where
        D: Send + 'static,
    {
        Self {
            data: Data::Async(
                app_data,
                Box::new(|repl, data| repl.eval_async(data).wait()),
            ),
            fmtrfn: None,
            resultfn: None,
            exitfn: None,
        }
    }
}

impl<'a, D, T, U, V> RunCallbacks<'a, D, T, U, V> {
    /// Specify code to be run which dictates the formatting configuration to use.
    pub fn with_fmtrfn<F>(self, f: F) -> RunCallbacks<'a, D, F, U, V>
    where
        F: FnMut(&Repl<Print, D>) -> FormattingConfig,
    {
        let RunCallbacks {
            data,
            resultfn,
            exitfn,
            ..
        } = self;
        RunCallbacks {
            data,
            fmtrfn: Some(f),
            resultfn,
            exitfn,
        }
    }

    /// Specify code to be run after evaluation has succeeded and a `Kserd` result is returned.
    ///
    /// The closure supplies the statement index `usize` and the result `Kserd`, along with the
    /// `Repl`.
    pub fn with_resultfn<F>(self, f: F) -> RunCallbacks<'a, D, T, F, V>
    where
        F: FnMut(usize, Kserd<'static>, &Repl<Read, D>),
    {
        let RunCallbacks {
            data,
            fmtrfn,
            exitfn,
            ..
        } = self;
        RunCallbacks {
            data,
            fmtrfn,
            resultfn: Some(f),
            exitfn,
        }
    }

    /// Specify code to be run after the exit signal is received.
    ///
    /// This can be used to clean up resources within [`ReplData`] or `D`.
    pub fn with_exitfn<F>(self, f: F) -> RunCallbacks<'a, D, T, U, F>
    where
        F: FnOnce(&ReplData<D>, &mut D),
    {
        let RunCallbacks {
            data,
            fmtrfn,
            resultfn,
            ..
        } = self;
        RunCallbacks {
            data,
            fmtrfn,
            resultfn,
            exitfn: Some(f),
        }
    }
}

/// Available with the `runnable` feature and when the REPL is in the `Read` state.
impl<D> Repl<Read, D> {
    /// Run the repl inside the terminal, consuming the repl. Returns the output of the REPL.
    pub fn run<T, U, V>(self, run_callbacks: RunCallbacks<D, T, U, V>) -> io::Result<String>
    where
        T: FnMut(&Repl<Print, D>) -> kserd::fmt::FormattingConfig,
        U: FnMut(usize, kserd::Kserd<'static>, &Repl<Read, D>),
        V: FnOnce(&mut ReplData<D>, &mut D),
    {
        run(self, run_callbacks, Screen::new).map_err(|e| map_xterm_err(e, "running REPL failed"))
    }
}

fn run<D, FmtrFn, ResultFn, ExitFn>(
    mut read: Repl<Read, D>,
    mut runcb: RunCallbacks<D, FmtrFn, ResultFn, ExitFn>,
    screen_fn: impl FnOnce() -> io::Result<Screen>,
) -> xterm::Result<String>
where
    FmtrFn: FnMut(&Repl<Print, D>) -> kserd::fmt::FormattingConfig,
    ResultFn: FnMut(usize, kserd::Kserd<'static>, &Repl<Read, D>),
    ExitFn: FnOnce(&mut ReplData<D>, &mut D),
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

    let mut screen = screen_fn()?;
    let mut inputbuf = interface::InputBuffer::new();
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

    // must seed the history size, this is maintained as constant.
    let mut history = std::collections::VecDeque::from(vec![String::default(); 100]); // 100 history

    let output = loop {
        let mut interface = screen.begin_interface_input(&mut inputbuf, &mut history)?;
        interface.set_prompt(&read.prompt(true));

        if let Some(buf) = read.data.editing_src.take() {
            if reevaluate.is_none() {
                interface.write(&buf);
            }
        }

        interface.flush_buffer()?;

        if let Some(val) = reevaluate.take() {
            read.line_input(&val);
        } else if do_read(&mut read, &mut interface, &cache)? {
            break read.output().to_owned();
        }

        match read.read() {
            ReadResult::Read(repl) => read = repl,
            ReadResult::Eval(repl) => {
                match do_eval(repl, &mut runcb) {
                    (mut repl, Signal::Exit) => {
                        // run exit function
                        if let Some(exitfn) = runcb.exitfn {
                            match runcb.data {
                                Data::Sync(d, _) => exitfn(&mut repl.data, d),
                                Data::Async(d, _) => exitfn(&mut repl.data, &mut d.lock().unwrap()),
                            }
                        }

                        break repl.output().to_owned();
                    }
                    (repl, signal) => {
                        let mut reeval = None;
                        if let Signal::ReEvaluate(s) = signal {
                            reeval = Some(s);
                        }
                        read = repl;
                        reevaluate = reeval;
                        // prep for next read
                        interface::erase_current_line(io::stdout())?.flush()?;
                    }
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
    interface: &mut Interface,
    cache: &CacheWrapper,
) -> xterm::Result<bool> {
    #[cfg(not(feature = "racer-completion"))]
    let _ = cache;
    
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
    const ENTER_VERBATIM_MODE: Event = Key(KeyEvent {
        modifiers: KeyModifiers::CONTROL,
        code: Char('o'),
    });
    const STOP_VERBATIM_MODE: Event = Key(KeyEvent {
        modifiers: KeyModifiers::CONTROL,
        code: Char('d'),
    });
    const STOPEVENTS: &[Event] = &[ENTER, TAB, BREAK, ENTER_VERBATIM_MODE, STOP_VERBATIM_MODE];

    let mut completion_writer = interface::CompletionWriter::new();
    let mut verbatim_mode = false;
    let rdata = &repl.data;
    let treecmpltr = TreeCompleter::build(&rdata.cmdtree);
    let modscmpltr = ModulesCompleter::build(&rdata.cmdtree, rdata.mods_map());
    #[cfg(feature = "racer-completion")]
    let codecmpltr = CodeCompleter::build(rdata);
    let prompt = repl.prompt(true);
    let verbatim_prompt = format!("{}\u{1b}[44m ", &prompt[..prompt.len() - 1]);

    loop {
        if verbatim_mode {
            interface.set_prompt(&verbatim_prompt);
        } else {
            interface.set_prompt(&prompt);
        }
        interface.flush_buffer()?;

        let ev = interface.read_until(STOPEVENTS)?;

        match (ev, verbatim_mode) {
            (ENTER, false) | (STOP_VERBATIM_MODE, true) => {
                let line = interface.buffer();
                repl.line_input(&line);
                interface.add_history(line);
                interface.mv_bufpos_end();
                interface.writeln("");
                interface.flush_buffer()?;
                break Ok(false);
            }
            (TAB, false) => {
                let line = interface.buffer();
                if completion_writer.is_same_input(&line) {
                    completion_writer.next_completion();
                } else {
                    let f = |start| {
                        interface
                            .buf_ch_len()
                            .saturating_sub(line[start..].chars().count())
                    };

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

                completion_writer.overwrite_completion(interface)?;
            }
            (BREAK, _) => break Ok(true),
            (ENTER_VERBATIM_MODE, false) => verbatim_mode = true,
            (ENTER, true) => {
                interface.writeln("");
                interface.flush_buffer()?;
            }
            (TAB, true) => {
                interface.write("\t");
                interface.flush_buffer()?;
            }
            _ => (), // do nothing otherwise
        }
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

fn do_eval<D, FmtrFn, ResultFn, ExitFn>(
    mut repl: Repl<Evaluate, D>,
    runcb: &mut RunCallbacks<D, FmtrFn, ResultFn, ExitFn>,
) -> (Repl<Read, D>, Signal)
where
    FmtrFn: FnMut(&Repl<Print, D>) -> kserd::fmt::FormattingConfig,
    ResultFn: FnMut(usize, kserd::Kserd<'static>, &Repl<Read, D>),
{
    let rx = repl.output_listen();

    let jh = std::thread::spawn(move || {
        let mut covered_lines = 0;
        for chg in rx.iter() {
            covered_lines = interface::write_output_chg(covered_lines, chg).unwrap_or(0);
        }
    });

    let r = match &mut runcb.data {
        Data::Sync(d, evalfn) => evalfn(repl, d),
        Data::Async(d, evalfn) => evalfn(repl, &d),
    };

    // prepare the formatter for output
    let fmt = runcb
        .fmtrfn
        .as_mut()
        .map(|f| f(&r.repl))
        .unwrap_or_default();

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

    (read, signal)
}

fn map_xterm_err(xtermerr: crossterm::ErrorKind, msg: &str) -> io::Error {
    match xtermerr {
        crossterm::ErrorKind::IoError(e) => e,
        _ => io::Error::new(io::ErrorKind::Other, msg),
    }
}
