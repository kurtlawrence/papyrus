use super::*;
use ::kserd::fmt::FormattingConfig;

/// > **These methods are available when the REPL is in the [`Print`] state.**
impl<D> Repl<Print, D> {
    /// Prints the result if successful as `[out#]` or the failure message if any.
    /// Uses the default formatter for the `Kserd` data.
    pub fn print(self) -> (Repl<Read, D>, Option<(usize, Kserd<'static>)>) {
        self.print_with_formatting(FormattingConfig::default())
    }

    /// Prints the result if successful as `[out#]` or the failure message if any.
    /// Uses the given formatting configuration for the `Kserd` data.
    /// The return is (<repl in read state>, <maybe <stmt index, data>>)
    pub fn print_with_formatting(
        self,
        config: FormattingConfig,
    ) -> (Repl<Read, D>, Option<(usize, Kserd<'static>)>) {
        let Repl {
            state,
            data,
            more,
            data_mrker,
        } = self;

        let repl_data = data;

        let Print { mut output, data } = state;

        let mut kserd = None;

        match data {
            EvalOutput::Data(k) => {
                let num = repl_data.current_src().stmts.len().saturating_sub(1);

                let out_stmt = format!("[out{}]", num);

                let line = format!(
                    "{} {}: {}",
                    repl_data.cmdtree.path().color(repl_data.prompt_colour),
                    out_stmt.color(repl_data.out_colour),
                    k.as_str_with_config(config)
                );

                output.write_line(&line);

                kserd = Some((num, k));
            }
            EvalOutput::Print(print) => {
                if print.len() > 0 {
                    // only write if there is something to write.
                    output.write_line(&print);
                }
            }
        }

        let mut r = Repl {
            state: Read {
                output: output.into_read(),
            },
            data: repl_data,
            data_mrker,
            more,
        };

        prepare_read(&mut r);

        (r, kserd)
    }
}

fn prepare_read<D>(repl: &mut Repl<Read, D>) {
    repl.draw_prompt();

    let editing_src = repl.data.editing.and_then(|ei| {
        let src = repl.data.current_src();

        match ei.editing {
            Editing::Crate => src.crates.get(ei.index).map(|x| &x.src_line).cloned(),
            Editing::Item => src.items.get(ei.index).map(|x| x.0.clone()),
            Editing::Stmt => src.stmts.get(ei.index).map(|x| x.src_line()),
        }
    });
    repl.data.editing_src = editing_src;
}
