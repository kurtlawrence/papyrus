#[cfg(feature = "racer-completion")]
use crate::complete::code::{CodeCache, CodeCompleter};
use crate::complete::{cmdr::TreeCompleter, modules::ModulesCompleter};
use crate::prelude::*;
use crossterm::ExecutableCommand;
use mortal::Event;
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

impl<D> Repl<Read, D> {
    /// Run the repl inside the terminal, consuming the repl. Returns the output of the REPL.
    pub fn run(self, app_data: &mut D) -> io::Result<String> {
        run(self, |repl| repl.eval(app_data))
    }

    /// Run the repl inside the terminal, consuming the repl. Returns the output of the REPL.
    /// Takes an `Arc<Mutex<D>>` for data and only locks on evaluation cycles.
    pub fn run_async(self, app_data: Arc<Mutex<D>>) -> io::Result<String>
    where
        D: 'static + Send,
    {
        run(self, |repl| repl.eval_async(&app_data).wait())
    }
}

fn run<D, F: FnMut(Repl<Evaluate, D>) -> EvalResult<D>>(
    mut read: Repl<Read, D>,
    evalfn: F,
) -> io::Result<String> {
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

    let mut boxedfn = Box::new(evalfn) as Box<dyn FnMut(Repl<Evaluate, D>) -> EvalResult<D>>;

    let output = loop {
        io::stdout().execute(crossterm::Output(read.prompt())).ok();

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
            ReadResult::Eval(repl) => match do_eval(repl, &mut boxedfn) {
                Ok((repl, reeval)) => {
                    read = repl;
                    reevaluate = reeval;

                    // prep for next read
                    interface::erase_current_line(io::stdout())?.flush()?;
                }
                Err(r) => break r.output().to_owned(),
            },
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
    use mortal::{Event::*, Key::*, Signal::*};
    const BREAK: Event = Signal(Interrupt);
    const ENTER: Event = Key(Enter);
    const TAB: Event = Key(Tab);
    const STOPEVENTS: &[Event] = &[BREAK, ENTER, TAB];

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

        if ev == BREAK {
            return Ok(true);
        } else if ev == ENTER {
            repl.line_input(&input.buffer());
            writeln!(&mut io::stdout(), "")?;
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
                    let injection = format!("{}\n{}", repl.input_buffer(), line);
                    #[cfg(feature = "racer-completion")]
                    let c = complete_code(&codecmpltr, &cache.0, &injection, code_chpos);

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

fn do_eval<'a, D>(
    mut repl: Repl<Evaluate, D>,
    evalfn: &mut Box<dyn FnMut(Repl<Evaluate, D>) -> EvalResult<D> + 'a>,
) -> Result<(Repl<Read, D>, Option<String>), Repl<Read, D>> {
    let rx = repl.output_listen();

    let jh = std::thread::spawn(move || {
        rx.iter()
            .for_each(|x| interface::write_output_chg(x).unwrap_or(()))
    });

    let r = evalfn(repl);
    let (mut read, signal) = (r.repl.print().0, r.signal);

    read.close_channel();
    jh.join().unwrap();

    match signal {
        Signal::None => Ok((read, None)),
        Signal::Exit => Err(read),
        Signal::ReEvaluate(val) => Ok((read, Some(val))),
    }
}
