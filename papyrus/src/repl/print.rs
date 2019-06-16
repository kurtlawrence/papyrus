use super::*;

use std::io::Write;

impl<D> Repl<Print, D> {
    /// Prints the result if successful as `[out#]` or the failure message if any.
    pub fn print(mut self) -> Repl<Read, D> {
        // write
        {
            if self.state.as_out {
                let num = self.data.current_src().stmts.len().saturating_sub(1);

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

        prepare_read(&mut r);

        r
    }
}

fn prepare_read<D>(repl: &mut Repl<Read, D>) {
    repl.draw_prompt();

    let editing_src = repl.data.editing.and_then(|ei| {
        let src = repl.data.current_src();

        match ei.editing {
            Editing::Crate => src.crates.get(ei.index).map(|x| &x.src_line).cloned(),
            Editing::Item => src.items.get(ei.index).cloned(),
            Editing::Stmt => src.stmts.get(ei.index).map(|x| x.src_line()),
        }
    });
    repl.data.editing_src = editing_src;
}
