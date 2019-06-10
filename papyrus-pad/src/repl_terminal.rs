use super::*;
use azul::app::AppStateNoData;
use azul::callbacks::DefaultCallback;
use completion::*;
use papyrus::prelude::*;
use repl::ReadResult;
use std::borrow::BorrowMut;

const SPACE: char = ' ';

struct InputHandled {
    redraw: UpdateScreen,
    start_eval: bool,
    start_complete: bool,
    hide_complete: bool,
}

impl Default for InputHandled {
    fn default() -> Self {
        Self {
            redraw: DontRedraw,
            start_eval: false,
            start_complete: false,
            hide_complete: false,
        }
    }
}

enum Input {
    Backspace,
    Char(char),
    Ctrl(char),
    Down,
    Return,
    Up,
}

impl<T, D> PadState<T, D>
where
    T: 'static + BorrowMut<AppValue<PadState<T, D>>>,
    D: 'static + Send + Sync,
{
    fn handle_input(
        &mut self,
        input: Input,
        app: &mut AppStateNoData<T>,
        event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let redraw = match input {
            Input::Backspace => {
                self.input_buffer.pop();
                self.set_repl_line_input();
                self.completion.last_completions.clear();
                Redraw
            }
            Input::Char(ch) => {
                self.input_buffer.push(ch);
                self.set_repl_line_input();
                self.start_completion_timer(app);
                Redraw
            }
            Input::Ctrl(ch) => match ch {
                SPACE => Some(self.start_completion_timer(app)),
                _ => DontRedraw,
            },
            Input::Down => {
                if !self.completion.will_render() {
                    DontRedraw
                } else {
                    DontRedraw
                }
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
                                self.repl.put_eval(r.eval_async(&self.data));
                                self.start_eval_timer(app);
                            }
                        }
                        Redraw
                    }
                    None => DontRedraw,
                }
            }
            Input::Up => {
                if !self.completion.will_render() {
                    DontRedraw
                } else {
                    DontRedraw
                }
            }
        };

        if redraw.is_some() {
            self.term_render.handle_line_changes(app.resources);
        }

        redraw
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

    fn start_completion_timer(&mut self, app_state: &mut AppStateNoData<T>) {
        if let Some(repl) = self.repl.brw_repl() {
            self.completion
                .to_complete(repl.input_buffer().to_owned(), None);
            let timer = Timer::new(Self::redraw_completions)
                .with_interval(std::time::Duration::from_millis(10));
            app_state.add_timer(self.completion_timer_id, timer);
        }
    }

    fn start_eval_timer(&self, app_state: &mut AppStateNoData<T>) {
        let timer = Timer::new(Self::check_evaluating_done)
            .with_interval(std::time::Duration::from_millis(10));
        app_state.add_timer(self.eval_timer_id, timer);
    }

    fn check_evaluating_done(
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

    fn redraw_completions(app: &mut T, _: &mut AppResources) -> (UpdateScreen, TerminateTimer) {
        let pad: &mut PadState<T, D> = &mut app.borrow_mut();

        if pad.completion.update() {
            (Redraw, TerminateTimer::Terminate)
        } else {
            (DontRedraw, TerminateTimer::Continue)
        }
    }

    fn update_state_on_text_input(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let kb = app_state.windows[window_event.window_id].get_keyboard_state();

        if kb.ctrl_down || kb.alt_down || kb.super_down {
            None
        } else {
            self.handle_input(Input::Char(kb.current_char?), app_state, window_event)
        }
    }

    fn update_state_on_vk_down(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        use AcceleratorKey::*;
        use VirtualKeyCode::*;

        let kb = app_state.windows[window_event.window_id].get_keyboard_state();

        let input = kb_seq(kb, &[Ctrl, Key(Space)], || Input::Ctrl(' '))
            .or_else(|| kb_seq(kb, &[Key(Back)], || Input::Backspace))
            .or_else(|| kb_seq(kb, &[Key(Return)], || Input::Return))
            .or_else(|| kb_seq(kb, &[Key(Up)], || Input::Up))
            .or_else(|| kb_seq(kb, &[Key(Down)], || Input::Down));

        input
            .and_then(|input| self.handle_input(input, app_state, window_event))
            .or_else(|| self.completion.on_focus_vk_down(app_state, window_event))
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
    pub fn dom<T, D>(state: &AppValue<PadState<T, D>>, info: &mut LayoutInfo<T>) -> Dom<T>
    where
        T: 'static + BorrowMut<AppValue<PadState<T, D>>>,
        D: 'static + Send + Sync,
    {
        let ptr = StackCheckedPointer::new(state);

        let text_input_cb_id = info.window.add_callback(
            ptr.clone(),
            DefaultCallback(PadState::priv_update_state_on_text_input),
        );
        let vk_down_cb_id = info.window.add_callback(
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

        // Completion
        if let Some(prompt) = CompletionPrompt::dom(&state.completion, info) {
            term_div.add_child(prompt);
        }

        term_div
    }
}
