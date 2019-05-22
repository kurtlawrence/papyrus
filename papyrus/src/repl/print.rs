use super::*;

use linefeed::terminal::Terminal;
use std::io::Write;

impl<Term: Terminal, Data> Repl<Print, Term, Data> {
    /// Prints the result if successful as `[out#]` or the failure message if any.
    pub fn print(self) -> Repl<Read, Term, Data> {
        // write
        {
            if self.state.as_out {
                let num = self
                    .data
                    .file_map
                    .get(&self.data.current_file)
                    .expect("file map does not contain key")
                    .stmts
                    .len()
                    .saturating_sub(1);
                let out_stmt = format!("[out{}]", num);
                writeln!(
                    Writer(self.terminal.terminal.as_ref()),
                    "{} {}: {}",
                    self.data.cmdtree.path().color(self.data.prompt_colour),
                    out_stmt.color(self.data.out_colour),
                    self.state.to_print
                )
                .expect("failed writing");
            } else {
                if self.state.to_print.len() > 0 {
                    // only write if there is something to write.
                    writeln!(
                        Writer(self.terminal.terminal.as_ref()),
                        "{}",
                        self.state.to_print
                    )
                    .expect("failed writing");
                }
            }
        }

        let r = self.move_state(Read);

        r.draw_prompt().unwrap(); // prep for next read

        r
    }
}
