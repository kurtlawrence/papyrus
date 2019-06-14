use super::*;
use eval_state::EvalState;
use papyrus::prelude::*;
use std::sync::{Arc, RwLock};

impl<T, D> PadState<T, D> {
    pub fn new(mut repl: Repl<repl::Read, D>, data: Arc<RwLock<D>>) -> Self {
        let term_render = ansi_renderer::ReplOutputRenderer::new(repl.output_listen());

        let completion = completion::CompletionPromptState::initialise(&repl.data);
        let completion = AppValue::new(completion);

        Self {
            repl: EvalState::new(repl),
            input_buffer: String::new(),
            history: history::History::new(),
            eval_timer_id: TimerId::new(),
            data,
            after_eval_fn: none,
            term_render,
            completion,
            completion_timer_id: TimerId::new(),
        }
    }

    pub fn with_after_eval_fn(mut self, func: fn(&mut T, &mut AppResources)) -> Self {
        self.after_eval_fn = func;
        self
    }

    /// Functions to run after the evaluation phase finished.
    pub fn eval_finished(&mut self) {
        if let Some(repl) = self.repl.brw_read() {
            self.completion.build_completers(&repl.data);

            if let Some(src) = repl.editing_src() {
                self.set_line_input(src);
            }
        }
    }

    pub fn set_line_input(&mut self, line: String) -> UpdateScreen {
        self.input_buffer = line;

        self.repl.take_read().map(|mut repl| {
            repl.line_input(&self.input_buffer);
            self.repl.put_read(repl);
        })
    }
}

fn none<T>(_: &mut T, _: &mut AppResources) {}
