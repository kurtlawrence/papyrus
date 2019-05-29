use super::*;
use azul::app::AppStateNoData;
use azul::callbacks::{DefaultCallback, DefaultCallbackId};
use azul::prelude::*;
use azul::window::FakeWindow;
use papyrus::prelude::*;
use std::borrow::BorrowMut;
use std::marker::PhantomData;

type KickOffEvalDaemon = bool;
type HandleCb = (UpdateScreen, KickOffEvalDaemon);

impl<T, D> PadState<T, D>
where
    D: 'static + Send + Sync,
{
    fn handle_input(&mut self, input: char) -> HandleCb {
        let mut kickoff = false;
        match self.repl.take_read() {
            Some(repl) => {
                match repl.push_input(input) {
                    repl::PushResult::Read(r) => self.repl.put_read(r),
                    repl::PushResult::Eval(r) => {
                        kickoff = true;
                        self.repl.put_eval(r.eval_async(&self.data));
                    }
                }
                (Redraw, kickoff)
            }
            None => (DontRedraw, kickoff),
        }
    }

    fn handle_vk(&mut self, vk: VirtualKeyCode) -> HandleCb {
        match vk {
            VirtualKeyCode::Back => self.handle_input('\x08'), // backspace character
            VirtualKeyCode::Tab => self.handle_input('\t'),
            VirtualKeyCode::Return => self.handle_input('\n'),
            _ => (DontRedraw, false),
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
        let (update_screen, kickoff) = self.handle_input(
            app_state.windows[window_event.window_id]
                .get_keyboard_state()
                .current_char?,
        );

        if kickoff {
            kickoff_daemon(app_state, self.eval_daemon_id);
        }

        update_screen
    }

    fn update_state_on_vk_down(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        let (update_screen, kickoff) = self.handle_vk(
            app_state.windows[window_event.window_id]
                .get_keyboard_state()
                .latest_virtual_keycode?,
        );

        if kickoff {
            kickoff_daemon(app_state, self.eval_daemon_id);
        }

        update_screen
    }
}

pub struct ReplTerminal<T, D> {
    text_input_cb_id: DefaultCallbackId,
    vk_down_cb_id: DefaultCallbackId,
    mrkr: PhantomData<T>,
    mrkr_data: PhantomData<D>,
}

impl<T, D> ReplTerminal<T, D>
where
    T: 'static + BorrowMut<AppValue<PadState<T, D>>>,
    D: 'static + Send + Sync,
{
    pub fn dom(state: &AppValue<PadState<T, D>>, window: &mut FakeWindow<T>) -> Dom<T> {
        let ptr = StackCheckedPointer::new(state);

        let text_input_cb_id = window.add_callback(
            ptr.clone(),
            DefaultCallback(Self::update_state_on_text_input),
        );
        let vk_down_cb_id =
            window.add_callback(ptr.clone(), DefaultCallback(Self::update_state_on_vk_down));

        let mut container = Dom::div()
            .with_class("repl-terminal")
            .with_tab_index(TabIndex::Auto); // make focusable
        container.add_default_callback_id(On::TextInput, text_input_cb_id);
        container.add_default_callback_id(On::VirtualKeyDown, vk_down_cb_id);

        let mut text = String::with_capacity(state.last_terminal_string.len());
        create_terminal_string(&state.terminal, &mut text);
        add_terminal_text(container, &text)
    }

    cb!(PadState, update_state_on_text_input);
    cb!(PadState, update_state_on_vk_down);
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
                // execute the after_eval_fn that is stored on pad
                (Redraw, TerminateTimer::Terminate, true) // turn off daemon now
            } else {
                pad.repl.put_eval(eval);
                (redraw_on_term_chg(pad), TerminateTimer::Continue, false) // continue to check later
            }
        }
        None => (DontRedraw, TerminateTimer::Terminate, false), // if there is no eval, may as well stop checking
    };

    if finished {
        (pad.after_eval_fn)(app, app_resources);
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
