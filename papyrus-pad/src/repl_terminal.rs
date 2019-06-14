use super::*;
use azul::app::AppStateNoData;
use azul::callbacks::DefaultCallback;
use completion::*;
use papyrus::prelude::*;
use repl::ReadResult;
use std::borrow::BorrowMut;

const WINDOW_LMD: EventFilter = EventFilter::Window(WindowEventFilter::LeftMouseDown);

const SPACE: char = ' ';
const LINE_HEIGHT: f32 = 25.0; // px

enum Input {
    Backspace,
    Char(char),
    Ctrl(char),
    Down,
    LeftMouseDown,
    Return,
    Tab,
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
        _: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let redraw = match input {
            Input::Backspace => {
                self.input_buffer.pop();
                self.set_repl_line_input();
                self.completion.clear();
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
                    if let Some(buf) = self.history.move_forwards() {
                        self.input_buffer.clear();
                        self.input_buffer.push_str(buf);
                    } else {
                        self.input_buffer.clear();
                    }

                    self.set_repl_line_input();
                    Redraw
                } else {
                    DontRedraw
                }
            }
            Input::LeftMouseDown => {
                if let Some(idx) = self.completion.last_mouse_hovered.as_ref().cloned() {
                    self.complete_input_buffer(idx)
                } else {
                    DontRedraw
                }
            }
            Input::Return => {
                self.completion.clear();

                self.history.add_unique(self.input_buffer.clone());
                self.history.reset_position();

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
            Input::Tab => self.complete_input_buffer(self.completion.kb_focus),
            Input::Up => {
                if !self.completion.will_render() {
                    if let Some(buf) = self.history.move_backwards() {
                        self.input_buffer.clear();
                        self.input_buffer.push_str(buf);
                        self.set_repl_line_input();
                        Redraw
                    } else {
                        DontRedraw
                    }
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

    fn complete_input_buffer(&mut self, idx: usize) -> UpdateScreen {
        if let Some(item) = self.completion.complete_input_buffer_line(idx) {
            self.input_buffer = item;
            self.set_repl_line_input();
            Redraw
        } else {
            DontRedraw
        }
    }

    fn start_completion_timer(&mut self, app_state: &mut AppStateNoData<T>) {
        if let Some(repl) = self.repl.brw_repl() {
            self.completion.to_complete(
                repl.input_buffer().to_owned(),
                repl.input_buffer_line().to_owned(),
                None,
            );
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
            .or_else(|| kb_seq(kb, &[Key(Down)], || Input::Down))
            .or_else(|| kb_seq(kb, &[Key(Tab)], || Input::Tab));

        input
            .and_then(|input| self.handle_input(input, app_state, window_event))
            .or_else(|| self.completion.on_focus_vk_down(app_state, window_event))
    }

    fn on_window_left_mouse_down(
        &mut self,
        app: &mut AppStateNoData<T>,
        event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        self.handle_input(Input::LeftMouseDown, app, event)
    }

    cb!(priv_update_state_on_text_input, update_state_on_text_input);
    cb!(priv_update_state_on_vk_down, update_state_on_vk_down);
    cb!(priv_on_window_left_mouse_down, on_window_left_mouse_down);
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
        let window_lmd_cb_id = info.window.add_callback(
            ptr.clone(),
            DefaultCallback(PadState::priv_on_window_left_mouse_down),
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
        term_div.add_default_callback_id(WINDOW_LMD, window_lmd_cb_id);

        // Rendered Output
        let output = state.term_render.dom();
        term_div.add_child(output);

        // Completion
        let top = state.term_render.lines.len() as f32 * LINE_HEIGHT;
        let left = 0.0;

        if let Some(prompt) = CompletionPrompt::dom(&state.completion, info, top, left) {
            term_div.add_child(prompt);
        }

        term_div
    }
}
