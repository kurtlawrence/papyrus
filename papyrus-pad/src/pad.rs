use super::*;
use eval_state::EvalState;
use papyrus::prelude::*;
use std::sync::{Arc, RwLock};

impl<T, D> PadState<T, D> {
    pub fn new(mut repl: Repl<repl::Read, D>, data: Arc<RwLock<D>>) -> Self {
        let term_render = ansi_renderer::ReplOutputRenderer::new(repl.output_listen());

        let completion = completion::CompletionPromptState::initialise(&repl.data);

        Self {
            repl: EvalState::new(repl),
            input_buffer: String::new(),
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
        if let Some(repl) = self.repl.brw_repl() {
            self.completion.build_completers(&repl.data);
        }
    }
}

fn none<T>(_: &mut T, _: &mut AppResources) {}
