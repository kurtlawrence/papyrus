use super::*;
use azul::prelude::*;
use eval_state::EvalState;
use papyrus::complete;
use papyrus::prelude::*;
use std::sync::{Arc, Mutex, RwLock};

impl<T, D> PadState<T, D> {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, D>, data: Arc<RwLock<D>>) -> Self {
        let term = repl.terminal().clone();

        let term_render = ansi_renderer::AnsiRenderer::new();

        let completion = completion::CompletionPromptState::new(&repl.data);

        Self {
            repl: EvalState::new(repl),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: TimerId::new(),
            data,
            after_eval_fn: none,
            term_render,
            completion,
        }
    }

    pub fn with_after_eval_fn(mut self, func: fn(&mut T, &mut AppResources)) -> Self {
        self.after_eval_fn = func;
        self
    }

    /// Functions to run after the evaluation phase finished.
    pub fn eval_finished(&mut self) {
        // if let Some(repl) = self.repl.brw_repl() {
        //     self.completers = completion::Completers::build(&repl.data);
        // }
    }

    pub fn initialise_resources(
        &mut self,
        app_resources: &mut AppResources,
    ) -> (UpdateScreen, TerminateTimer) {
        let mut s = String::new();
        create_terminal_string(&self.terminal, &mut s);

        self.term_render.update_text(&s, app_resources);

        self.last_terminal_string = s;

        (Redraw, TerminateTimer::Terminate)
    }
}

fn none<T>(_: &mut T, _: &mut AppResources) {}

pub fn initialise_resources_task<T>(cb: azul::callbacks::TimerCallbackType<T>) -> Task<T> {
    Task::new(&Arc::new(Mutex::new(())), initialise_resources_task_inner).then(Timer::new(cb))
}

fn initialise_resources_task_inner(_: Arc<Mutex<()>>, _: DropCheck) {}
