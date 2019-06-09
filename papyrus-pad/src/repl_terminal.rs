use super::*;
use azul::app::AppStateNoData;
use azul::callbacks::DefaultCallback;
use azul::window::FakeWindow;
use completion::*;
use papyrus::prelude::*;
use repl::ReadResult;
use std::borrow::BorrowMut;

struct InputHandled {
    redraw: UpdateScreen,
    start_eval: bool,
    start_complete: bool,
}

impl Default for InputHandled {
    fn default() -> Self {
        Self {
            redraw: DontRedraw,
            start_eval: false,
            start_complete: false,
        }
    }
}

enum Input {
    Backspace,
    Char(char),
    Return,
}

impl<T, D> PadState<T, D>
where
    T: 'static + BorrowMut<AppValue<PadState<T, D>>>,
    D: 'static + Send + Sync,
{
    fn handle_input(&mut self, input: Input) -> InputHandled {
        let mut handled = InputHandled::default();

        match input {
            Input::Backspace => {
                self.input_buffer.pop();
                self.set_repl_line_input();
                handled.redraw = Redraw;
            }
            Input::Char(ch) => {
                self.input_buffer.push(ch);
                self.set_repl_line_input();
                handled.start_complete = true;
                handled.redraw = Redraw;
            }
            Input::Return => {
                self.input_buffer.clear();

                match self.repl.take_read() {
                    Some(repl) => {
                        match repl.read() {
                            ReadResult::Read(r) => {
                                self.repl.put_read(r);
                            }
                            ReadResult::Eval(r) => {
                                handled.start_eval = true;
                                self.repl.put_eval(r.eval_async(&self.data));
                            }
                        }
                        handled.redraw = Redraw;
                    }
                    None => (),
                }
            }
        };

        handled
    }

    fn set_repl_line_input(&mut self) {
        match self.repl.take_read() {
            Some(mut repl) => {
                repl.line_input(&self.input_buffer);
                self.repl.put_read(repl);
            }
            None => (),
        }
    }

    fn update(&mut self, app_state: &mut AppStateNoData<T>, handled: InputHandled) -> UpdateScreen {
        if handled.redraw.is_some() {
            self.term_render.handle_line_changes(app_state.resources);
        }

        if handled.start_eval {
            let daemon = Timer::new(Self::check_evaluating_done)
                .with_interval(std::time::Duration::from_millis(10));
            app_state.add_timer(self.eval_daemon_id, daemon);
        }

        if handled.start_complete {
            if let Some(repl) = self.repl.brw_repl() {
                self.completion
                    .to_complete(repl.input_buffer().to_owned(), None);
            }
        }

        handled.redraw
    }

    pub fn check_evaluating_done(
        app: &mut T,
        app_resources: &mut AppResources,
    ) -> (UpdateScreen, TerminateTimer) {
        let pad: &mut PadState<T, D> = &mut app.borrow_mut();

        let (terminate, finished) = match pad.repl.take_eval() {
            Some(eval) => {
                if eval.completed() {
                    pad.repl.put_read(eval.wait().repl.print());
                    (TerminateTimer::Terminate, true) // turn off daemon now
                } else {
                    pad.repl.put_eval(eval);
                    (TerminateTimer::Continue, false) // continue to check later
                }
            }
            None => (TerminateTimer::Terminate, false), // if there is no eval, may as well stop checking
        };

        let redraw = pad.term_render.handle_line_changes(app_resources); // update any line changes no matter what

        if finished {
            // execute eval_finished on PadState
            pad.eval_finished();

            // execute the after_eval_fn that is stored on pad
            (pad.after_eval_fn)(app, app_resources) // run user defined after_eval_fn
        }

        if redraw || finished {
            (Redraw, terminate)
        } else {
            (DontRedraw, terminate)
        }
    }

    fn update_state_on_text_input(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let ch = app_state.windows[window_event.window_id]
            .get_keyboard_state()
            .current_char?;

        let hcb = self.handle_input(Input::Char(ch));

        self.update(app_state, hcb)
    }

    fn update_state_on_vk_down(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let hcb = match app_state.windows[window_event.window_id]
            .get_keyboard_state()
            .latest_virtual_keycode?
        {
            VirtualKeyCode::Back => self.handle_input(Input::Backspace),
            VirtualKeyCode::Return => self.handle_input(Input::Return),
            _ => InputHandled::default(),
        };

        self.update(app_state, hcb)
    }

    fn priv_update_state_on_text_input(
        data: &StackCheckedPointer<T>,
        app_state_no_data: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        data.invoke_mut(
            Self::update_state_on_text_input,
            app_state_no_data,
            window_event,
        )
    }

    fn priv_update_state_on_vk_down(
        data: &StackCheckedPointer<T>,
        app_state_no_data: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        data.invoke_mut(
            Self::update_state_on_vk_down,
            app_state_no_data,
            window_event,
        )
    }
}

pub struct ReplTerminal;

impl ReplTerminal {
    pub fn dom<T, D>(state: &AppValue<PadState<T, D>>, window: &mut FakeWindow<T>) -> Dom<T>
    where
        T: 'static + BorrowMut<AppValue<PadState<T, D>>>,
        D: 'static + Send + Sync,
    {
        let ptr = StackCheckedPointer::new(state);

        let text_input_cb_id = window.add_callback(
            ptr.clone(),
            DefaultCallback(PadState::priv_update_state_on_text_input),
        );
        let vk_down_cb_id = window.add_callback(
            ptr.clone(),
            DefaultCallback(PadState::priv_update_state_on_vk_down),
        );

        // Container Div
        let mut term_div = Dom::div()
            .with_class("repl-terminal")
            .with_tab_index(TabIndex::Auto); // make focusable

        term_div.add_default_callback_id(
            EventFilter::Focus(FocusEventFilter::TextInput),
            text_input_cb_id,
        );
        term_div.add_default_callback_id(
            EventFilter::Focus(FocusEventFilter::VirtualKeyDown),
            vk_down_cb_id,
        );

        // Rendered Output
        let output = state.term_render.dom();
        term_div.add_child(output);

        // term_div

        // Completion
        let mut container = Dom::div().with_child(term_div);

        let completions = state.completion.completions();

        if !completions.is_empty() {
            container.add_child(CompletionPrompt::dom(completions));
        }

        container
    }
}
