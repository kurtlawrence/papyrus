use super::*;
use azul::app::AppStateNoData;
use azul::callbacks::{DefaultCallback, DefaultCallbackId};
use azul::prelude::*;
use azul::window::FakeWindow;
use papyrus::prelude::*;
use std::borrow::BorrowMut;
use std::marker::PhantomData;
use std::sync::Mutex;

type KickOffEvalDaemon = bool;
type KickOffCompletionTask = bool;
type HandleCb = (UpdateScreen, KickOffEvalDaemon, KickOffCompletionTask);

impl<T, D> PadState<T, D>
where
    D: 'static + Send + Sync,
{
    fn handle_input(&mut self, input: char) -> HandleCb {
        let mut kickoff_eval = false;
        let mut kickoff_completion = false;

        match self.repl.take_read() {
            Some(repl) => {
                match repl.push_input(input) {
                    repl::PushResult::Read(r) => {
                        self.repl.put_read(r);
                        kickoff_completion = true;
                    }
                    repl::PushResult::Eval(r) => {
                        kickoff_eval = true;
                        self.repl.put_eval(r.eval_async(&self.data));
                    }
                }
                (Redraw, kickoff_eval, kickoff_completion)
            }
            None => (DontRedraw, kickoff_eval, kickoff_completion),
        }
    }

    fn handle_vk(&mut self, vk: VirtualKeyCode) -> HandleCb {
        match vk {
            VirtualKeyCode::Back => self.handle_input('\x08'), // backspace character
            VirtualKeyCode::Tab => self.handle_input('\t'),
            VirtualKeyCode::Return => self.handle_input('\n'),
            _ => (DontRedraw, false, false),
        }
    }
}

impl<T, D> PadState<T, D>
where
    T: BorrowMut<AppValue<PadState<T, D>>>,
    D: 'static + Send + Sync,
{
    fn update_state_on_text_input(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let (update_screen, kickoff_eval, kickoff_completion) = self.handle_input(
            app_state.windows[window_event.window_id]
                .get_keyboard_state()
                .current_char?,
        );

        if update_screen.is_some() {
            let mut buf = String::with_capacity(self.last_terminal_string.len());
            create_terminal_string(&self.terminal, &mut buf);
            self.term_render.update_text(&buf, app_state.resources);
        }

        if kickoff_eval {
            kickoff_daemon(app_state, self.eval_daemon_id);
        }

        if kickoff_completion {
            if let Some(repl) = self.repl.brw_repl() {
                app_state.add_task(self.completion.complete(repl.input(), None));
            }
        }

        update_screen
    }

    fn update_state_on_vk_down(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let (update_screen, kickoff_eval, kickoff_completion) = self.handle_vk(
            app_state.windows[window_event.window_id]
                .get_keyboard_state()
                .latest_virtual_keycode?,
        );

        if update_screen.is_some() {
            let mut buf = String::with_capacity(self.last_terminal_string.len());
            create_terminal_string(&self.terminal, &mut buf);
            self.term_render.update_text(&buf, app_state.resources);
        }

        if kickoff_eval {
            kickoff_daemon(app_state, self.eval_daemon_id);
        }

        if kickoff_completion {
            if let Some(repl) = self.repl.brw_repl() {
                app_state.add_task(self.completion.complete(repl.input(), None));
            }
        }

        update_screen
    }
}

impl<T, D> PadState<T, D>
where
    T: 'static + BorrowMut<AppValue<PadState<T, D>>>,
    D: 'static + Send + Sync,
{
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

        let mut container = Dom::div()
            .with_class("repl-terminal")
            .with_tab_index(TabIndex::Auto); // make focusable
        container.add_default_callback_id(On::TextInput, text_input_cb_id);
        container.add_default_callback_id(
            EventFilter::Focus(FocusEventFilter::VirtualKeyDown),
            vk_down_cb_id,
        );

        // let mut text = String::with_capacity(state.last_terminal_string.len());
        // create_terminal_string(&state.terminal, &mut text);

        // let container = add_terminal_text(container, &text);

        container.add_child(state.term_render.dom());

        let mut container = Dom::div().with_child(container);

        if let Ok(lock) = state.completion.data.try_lock() {
            let completions = &lock.completions;
            if !completions.is_empty() {
                container.add_child(completion::CompletionPrompt::dom(
                    state,
                    window,
                    completions.clone(),
                ));
            }
        }

        container
    }
}

fn kickoff_daemon<T, D>(app_state: &mut AppStateNoData<T>, daemon_id: TimerId)
where
    T: BorrowMut<AppValue<PadState<T, D>>>,
{
    let daemon =
        Timer::new(check_evaluating_done).with_interval(std::time::Duration::from_millis(2));
    app_state.add_timer(daemon_id, daemon);
}

fn check_evaluating_done<T, D>(
    app: &mut T,
    app_resources: &mut AppResources,
) -> (UpdateScreen, TerminateTimer)
where
    T: BorrowMut<AppValue<PadState<T, D>>>,
{
    // FIXME need to bench this and redraw_on_term_chg to check that it runs quickly
    // hopefully less than 1ms as it needs to be fast...

    let pad: &mut PadState<T, D> = &mut app.borrow_mut();

    let (redraw, terminate, finished) = match pad.repl.take_eval() {
        Some(eval) => {
            if eval.completed() {
                pad.repl.put_read(eval.wait().repl.print());
                (Redraw, TerminateTimer::Terminate, true) // turn off daemon now
            } else {
                pad.repl.put_eval(eval);
                (redraw_on_term_chg(pad), TerminateTimer::Continue, false) // continue to check later
            }
        }
        None => (DontRedraw, TerminateTimer::Terminate, false), // if there is no eval, may as well stop checking
    };

    if redraw.is_some() {
        let mut buf = String::with_capacity(pad.last_terminal_string.len());
        create_terminal_string(&pad.terminal, &mut buf);
        pad.term_render.update_text(&buf, app_resources);
    }

    if finished {
        // execute eval_finished on PadState
        pad.eval_finished();

        // execute the after_eval_fn that is stored on pad
        (pad.after_eval_fn)(app, app_resources) // run user defined after_eval_fn
    }

    (redraw, terminate)
}

fn redraw_on_term_chg<T, D>(pad: &mut PadState<T, D>) -> UpdateScreen {
    let mut new_str = String::with_capacity(pad.last_terminal_string.len());
    create_terminal_string(&pad.terminal, &mut new_str);
    if new_str != pad.last_terminal_string {
        pad.last_terminal_string = new_str;
        Redraw
    } else {
        DontRedraw
    }
}

/// Fills the buffer with the terminal contents. Clears buffer before writing.
pub fn create_terminal_string(term: &MemoryTerminal, buf: &mut String) {
    buf.clear();

    let mut lines = term.lines();
    while let Some(chars) = lines.next() {
        for ch in chars {
            buf.push(*ch);
        }
        buf.push('\n');
    }
}

fn colour_slice<T>(cat_slice: &cansi::CategorisedSlice) -> Dom<T> {
    const PROPERTY_STR: &str = "ansi_esc_color";
    let s = cat_slice.text.to_string();

    Dom::label(s)
        .with_class("repl-terminal-text")
        .with_css_override(
            PROPERTY_STR,
            StyleTextColor(crate::colour::map(&cat_slice.fg_colour)).into(),
        )
}

pub fn add_terminal_text<T>(mut container: Dom<T>, text: &str) -> Dom<T> {
    let categorised = cansi::categorise_text(text);

    for line in cansi::line_iter(&categorised) {
        let mut line_div = Dom::div().with_class("repl-terminal-line");
        for cat in line {
            line_div.add_child(colour_slice(&cat));
        }
        container.add_child(line_div);
    }

    container
}
