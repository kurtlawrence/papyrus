use super::*;

use std::io::Write;

impl<D> Repl<Print, D> {
    /// Prints the result if successful as `[out#]` or the failure message if any.
    pub fn print(mut self) -> Repl<Read, D> {
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
                    &mut self.state.output,
                    "{} {}: {}",
                    self.data.cmdtree.path().color(self.data.prompt_colour),
                    out_stmt.color(self.data.out_colour),
                    self.state.to_print
                )
                .expect("failed writing");
            } else {
                if self.state.to_print.len() > 0 {
                    // only write if there is something to write.
                    writeln!(&mut self.state.output, "{}", self.state.to_print)
                        .expect("failed writing");
                }
            }
        }

        let mut r = self.move_state(|s| Read {
            output: s.output.to_read(),
        });

        r.draw_prompt();

        r
    }
}
