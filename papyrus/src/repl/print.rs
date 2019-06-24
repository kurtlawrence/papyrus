use super::*;
use ::kserd::format::FormattingConfig;
use std::io::Write;

impl<D> Repl<Print, D> {
    /// Prints the result if successful as `[out#]` or the failure message if any.
    /// Uses the default formatter for the `Kserd` data.
    pub fn print(mut self) -> Repl<Read, D> {
        self.print_with_formatting(FormattingConfig::default())
    }

    pub fn print_with_formatting(mut self, config: FormattingConfig) -> Repl<Read, D> {
        match &self.state.data {
            EvalOutput::Data(kserd) => {
                let num = self.data.current_src().stmts.len().saturating_sub(1);

                let out_stmt = format!("[out{}]", num);

                writeln!(
                    &mut self.state.output,
                    "{} {}: {}",
                    self.data.cmdtree.path().color(self.data.prompt_colour),
                    out_stmt.color(self.data.out_colour),
                    kserd.as_str_with_config(config),
                )
                .expect("failed writing");
            }
            EvalOutput::Print(print) => {
                if print.len() > 0 {
                    // only write if there is something to write.
                    writeln!(&mut self.state.output, "{}", print).expect("failed writing");
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
