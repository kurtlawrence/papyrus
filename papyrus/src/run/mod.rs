#[cfg(feature = "racer-completion")]
use crate::complete::code::{CodeCache, CodeCompleter};
use crate::complete::{cmdr::TreeCompleter, modules::ModulesCompleter};
use crate::output;
use crate::prelude::*;
use crossterm::ExecutableCommand;
use mortal::{Event, Key};
use repl::{Evaluate, Read, ReadResult};
use std::cmp::max;
use std::io::{self, prelude::*};
use std::sync::Arc;

mod interface;

use interface::{InputBuffer, Screen};

impl<D> Repl<Read, D> {
    pub fn run(self, app_data: &mut D) -> io::Result<()> {
        run(self, app_data)
    }
}

fn run<D>(mut read: Repl<Read, D>, app_data: &mut D) -> io::Result<()> {
    let mut screen = interface::Screen::new()?;

    let mut reevaluate: Option<String> = None;

    loop {
        io::stdout().execute(crossterm::Output(read.prompt()));

        let mut input_buf = interface::InputBuffer::new();

        if let Some(buf) = read.data.editing_src.take() {
            if reevaluate.is_none() {
                input_buf.insert_str(&buf);
            }
        }

        if let Some(val) = reevaluate.take() {
            read.line_input(&val);
        } else {
            if do_read(&mut read, &mut screen, input_buf)? {
                break Ok(());
            }
        }

        match read.read() {
            ReadResult::Read(repl) => read = repl,
            ReadResult::Eval(repl) => match do_eval(repl, app_data) {
                Ok((repl, reeval)) => {
                    read = repl;
                    reevaluate = reeval;

                    // prep for next read
                    interface::erase_current_line(io::stdout()).flush()?;
                }
                Err(_) => break Ok(()),
            },
        }
    }
}

/// Returns true if interrupt occurred.
fn do_read<D>(repl: &mut Repl<Read, D>, screen: &mut Screen, buf: InputBuffer) -> io::Result<bool> {
    use mortal::{Event::*, Key::*, Signal::*};
    const BREAK: Event = Signal(Interrupt);
    const ENTER: Event = Key(Enter);
    const TAB: Event = Key(Tab);
    const STOPEVENTS: &[Event] = &[BREAK, ENTER, TAB];

    let rdata = &repl.data;
    let tree = TreeCompleter::build(&rdata.cmdtree);

    let mut i = Some(buf);

    let initial = crossterm::cursor().pos();

    let mut completion_writer = interface::CompletionWriter::new();

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
                let chpos_start = {
                    let start = TreeCompleter::word_break(&line);
                    let chars = line[start..].chars().count();
                    input.ch_len().saturating_sub(chars)
                };
                let completions = tree.complete(&line).map(|x| x.0.to_string());
                completion_writer.new_completions(completions, chpos_start);
            }

            completion_writer.overwrite_completion(initial, &mut input)?;
        }

        i = Some(input); // prep for next loop
    }
}

fn do_eval<D>(
    mut repl: Repl<Evaluate, D>,
    app_data: &mut D,
) -> Result<(Repl<Read, D>, Option<String>), ()> {
    let rx = repl.output_listen();

    let jh = std::thread::spawn(move || {
        rx.iter()
            .for_each(|x| interface::write_output_chg(x).unwrap_or(()))
    });

    let mut reevaluate = None;

    let result = repl.eval(app_data);

    match result.signal {
        Signal::None => (),
        Signal::Exit => return Err(()),
        Signal::ReEvaluate(val) => reevaluate = Some(val),
    }

    let mut read = result.repl.print().0;

    read.close_channel();
    jh.join().unwrap();

    Ok((read, reevaluate))
}

// struct Completer {
//     tree_cmplter: TreeCompleter,
//     mod_cmplter: ModulesCompleter,
//
//     #[cfg(feature = "racer-completion")]
//     code_cmplter: CodeCompleter,
//     //     #[cfg(feature = "racer-completion")]
//     //     code_cache: Arc<Mutex<CodeCache>>,
// }
//
// impl Completer {
//     #[cfg(feature = "racer-completion")]
//     fn build<T>(rdata: &repl::ReplData<T>) -> Self {
//         let tree_cmplter = TreeCompleter::build(&rdata.cmdtree);
//         let mod_cmplter = ModulesCompleter::build(&rdata.cmdtree, rdata.mods_map());
//         let code_cmplter = CodeCompleter::build(rdata);
//         let code_cache = CodeCache::new();
//
//         Self {
//             tree_cmplter,
//             mod_cmplter,
//             code_cmplter,
//             //             code_cache,
//         }
//     }
//
//     #[cfg(not(feature = "racer-completion"))]
//     fn build<T>(rdata: &repl::ReplData<T>) -> Self {
//         let tree_cmplter = TreeCompleter::build(&rdata.cmdtree);
//         let mod_cmplter = ModulesCompleter::build(&rdata.cmdtree, rdata.mods_map());
//
//         Self {
//             tree_cmplter,
//             mod_cmplter,
//         }
//     }
// }
//
// #[cfg(not(feature = "racer-completion"))]
// impl<T: Terminal> linefeed::Completer<T> for Completer {
//     fn complete(
//         &self,
//         word: &str,
//         prompter: &Prompter<T>,
//         _start: usize,
//         _end: usize,
//     ) -> Option<Vec<Completion>> {
//         let mut v = Vec::new();
//
//         let line = prompter.buffer();
//
//         let start = get_start(word, line);
//
//         v.extend(trees_completer(&self.tree_cmplter, line, start));
//
//         if let Some(mods) = complete_mods(&self.mod_cmplter, line) {
//             v.extend(mods);
//         }
//
//         if v.len() > 0 {
//             Some(v)
//         } else {
//             None
//         }
//     }
//
//     fn word_start(&self, line: &str, _end: usize, _prompter: &Prompter<T>) -> usize {
//         let s1 = TreeCompleter::word_break(line);
//         let s2 = ModulesCompleter::word_break(line);
//
//         max(s1, s2)
//     }
// }
//
// #[cfg(feature = "racer-completion")]
// impl<T: Terminal> linefeed::Completer<T> for Completer {
//     fn complete(
//         &self,
//         word: &str,
//         prompter: &Prompter<T>,
//         _start: usize,
//         _end: usize,
//     ) -> Option<Vec<Completion>> {
//         let mut v = Vec::new();
//
//         let line = prompter.buffer();
//
//         let start = get_start(word, line);
//
//         v.extend(trees_completer(&self.tree_cmplter, line, start));
//
//         if let Some(mods) = complete_mods(&self.mod_cmplter, line) {
//             v.extend(mods);
//         }
//
//         if !line.starts_with('.') {
//             let cache = CodeCache::new();
//             let code = self
//                 .code_cmplter
//                 .complete(line, Some(10), &cache)
//                 .into_iter()
//                 .map(|x| Completion {
//                     completion: x.matchstr,
//                     display: None,
//                     suffix: Suffix::None,
//                 });
//             v.extend(code);
//         }
//
//         if v.len() > 0 {
//             Some(v)
//         } else {
//             None
//         }
//     }
//
//     fn word_start(&self, line: &str, _end: usize, _prompter: &Prompter<T>) -> usize {
//         let s1 = TreeCompleter::word_break(line);
//         let s2 = ModulesCompleter::word_break(line);
//         let s3 = CodeCompleter::word_break(line);
//
//         max(max(s1, s2), s3)
//     }
// }
//
// fn get_start(word: &str, line: &str) -> usize {
//     let end = word.len() + 1;
//     if !word.is_empty() && line.len() >= end && &line[..1] == "." && &line[1..end] == word {
//         1
//     } else {
//         0
//     }
// }
//
// fn trees_completer<'a>(
//     cmpltr: &'a TreeCompleter,
//     line: &'a str,
//     start: usize,
// ) -> impl Iterator<Item = Completion> + 'a {
//     cmpltr
//         .complete(line)
//         .map(move |x| &x.0[start..])
//         .map(|x| Completion::simple(x.to_string()))
// }
//
// fn complete_mods<'a>(
//     cmpltr: &'a ModulesCompleter,
//     line: &'a str,
// ) -> Option<impl Iterator<Item = Completion> + 'a> {
//     cmpltr
//         .complete(line)
//         .map(|x| x.map(|y| Completion::simple(y)))
// }
