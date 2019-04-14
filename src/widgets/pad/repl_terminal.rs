use super::*;
use crate::prelude::*;
use crate::widgets;
use azul::app::AppStateNoData;
use azul::callbacks::{DefaultCallback, DefaultCallbackId};
use azul::prelude::*;
use azul::window::FakeWindow;
use linefeed::memory::MemoryTerminal;
use std::borrow::BorrowMut;
use std::marker::PhantomData;

type KickOffEvalDaemon = bool;
type HandleCb = (UpdateScreen, KickOffEvalDaemon);

impl<D: 'static + Send + Sync> PadState<D> {
    pub fn update_state_on_text_input<T: Layout + BorrowMut<AppValue<PadState<D>>>>(
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

    pub fn update_state_on_vk_down<T: Layout + BorrowMut<AppValue<PadState<D>>>>(
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

pub struct ReplTerminal<T: Layout, D> {
    text_input_cb_id: DefaultCallbackId,
    vk_down_cb_id: DefaultCallbackId,
    mrkr: PhantomData<T>,
    mrkr_data: PhantomData<D>,
}

impl<D: 'static + Send + Sync, T: Layout + BorrowMut<AppValue<PadState<D>>>> ReplTerminal<T, D> {
    pub fn new(
        window: &mut FakeWindow<T>,
        state_to_bind: &AppValue<PadState<D>>,
        full_data_model: &T,
    ) -> Self {
        let ptr = StackCheckedPointer::new(full_data_model, state_to_bind).unwrap();
        let text_input_cb_id =
            window.add_callback(ptr.clone(), DefaultCallback(Self::update_state_on_text_input));
        let vk_down_cb_id =
            window.add_callback(ptr.clone(), DefaultCallback(Self::update_state_on_vk_down));

        Self {
            text_input_cb_id,
            vk_down_cb_id,
            mrkr: PhantomData,
            mrkr_data: PhantomData,
        }
    }

    pub fn dom(self, state_to_render: &PadState<D>) -> Dom<T> {
        let term_str = create_terminal_string(&state_to_render.terminal);

        let categorised = cansi::categorise_text(&term_str);

        let mut container = Dom::div()
            .with_class("repl-terminal")
            .with_tab_index(TabIndex::Auto); // make focusable
        container.add_default_callback_id(On::TextInput, self.text_input_cb_id);
        container.add_default_callback_id(On::VirtualKeyDown, self.vk_down_cb_id);

        for line in cansi::line_iter(&categorised) {
            let mut line_div = Dom::div().with_class("repl-terminal-line");
            for cat in line {
                line_div.add_child(colour_slice(&cat));
            }
            container.add_child(line_div);
        }

        //container.debug_dump();	// debug layout

        container
    }

    cb!(PadState, update_state_on_text_input);
    cb!(PadState, update_state_on_vk_down);
}

fn kickoff_daemon<D, T: Layout + BorrowMut<AppValue<PadState<D>>>>(
    app_state: &mut AppStateNoData<T>,
    daemon_id: TimerId,
) {
    let daemon =
        Timer::new(check_evaluating_done).with_interval(std::time::Duration::from_millis(2));
    app_state.add_timer(daemon_id, daemon);
}

fn check_evaluating_done<D, T: BorrowMut<AppValue<PadState<D>>>>(
    app: &mut T,
    _: &mut AppResources,
) -> (UpdateScreen, TerminateTimer) {
    let pad: &mut PadState<D> = &mut app.borrow_mut().borrow_mut();

    match pad.repl.take_eval() {
        Some(eval) => {
            if eval.completed() {
                pad.repl.put_read(
                    eval.wait().repl.print(),
                );
                (Redraw, TerminateTimer::Terminate) // turn off daemon now
            } else {
                pad.repl.put_eval(eval);
                (redraw_on_term_chg(pad), TerminateTimer::Continue) // continue to check later
            }
        }
        None => (DontRedraw, TerminateTimer::Terminate), // if there is no eval, may as well stop checking
    }
}

fn redraw_on_term_chg<D>(pad: &mut PadState<D>) -> UpdateScreen {
    let new_str = create_terminal_string(&pad.terminal);
    if new_str != pad.last_terminal_string {
        pad.last_terminal_string = new_str;
        Redraw
    } else {
        DontRedraw
    }
}

fn create_terminal_string(term: &MemoryTerminal) -> String {
    let mut string = String::new();
    let mut lines = term.lines();
    while let Some(chars) = lines.next() {
        for ch in chars {
            string.push(*ch);
        }
        string.push('\n');
    }
    string
}

fn colour_slice<T: Layout>(cat_slice: &cansi::CategorisedSlice) -> Dom<T> {
    const PROPERTY_STR: &str = "ansi_esc_color";
    let s = String::from_utf8_lossy(cat_slice.text_as_bytes).to_string();

    Dom::label(s)
        .with_class("repl-terminal-text")
        .with_css_override(
            PROPERTY_STR,
            StyleTextColor(widgets::colour::map(&cat_slice.fg_colour)).into(),
        )
}

