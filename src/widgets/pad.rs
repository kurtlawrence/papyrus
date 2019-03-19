use crate::prelude::*;
use crate::widgets;
use azul::app_state::AppStateNoData;
use azul::default_callbacks::{DefaultCallback, DefaultCallbackId};
use azul::prelude::*;
use azul::window::FakeWindow;
use linefeed::memory::MemoryTerminal;
use std::marker::PhantomData;
use super::*;

pub struct PadState {
    terminal: MemoryTerminal,
    repl: Option<Repl<repl::Read, MemoryTerminal, (), linking::NoRef>>,
    eval: Option<repl::Evaluating<MemoryTerminal, (), linking::NoRef>>,
}

impl PadState {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, (), linking::NoRef>) -> Self {
        PadState {
            terminal: repl.terminal_inner().clone(),
            repl: Some(repl),
            eval: None,
        }
    }

    pub fn update_state_on_text_input<T: Layout>(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
		self.handle_input(app_state.windows[window_event.window_id].get_keyboard_state().current_char?)
    }

	pub fn update_state_on_vk_down<T: Layout>(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
		self.handle_vk(app_state.windows[window_event.window_id].get_keyboard_state().latest_virtual_keycode?)
    }

    fn handle_input(&mut self, input: char) -> UpdateScreen {
        if self.eval.is_none() {
            let repl = self
                .repl
                .take()
                .expect("repl was empty, which would indicate a broken state?");
            match repl.push_input(input) {
                repl::PushResult::Read(r) => self.repl = Some(r),
                repl::PushResult::Eval(r) => self.eval = Some(r.eval_async(())),
            }
            Redraw
        } else {
            DontRedraw
        }
    }

    fn handle_vk(&mut self, vk: VirtualKeyCode) -> UpdateScreen {
        match vk {
            VirtualKeyCode::Back => self.handle_input('\x08'), // backspace character
            VirtualKeyCode::Tab => self.handle_input('\t'),
            VirtualKeyCode::Return => self.handle_input('\n'),
            _ => DontRedraw,
        }
    }
}

pub struct Pad<T: Layout> {
    text_input_cb_id: DefaultCallbackId,
	vk_down_cb_id: DefaultCallbackId,
    mrkr: PhantomData<T>,
}

impl<T: Layout> Pad<T> {
    pub fn new(window: &mut FakeWindow<T>, state_to_bind: &PadState, full_data_model: &T) -> Self {
        let ptr = StackCheckedPointer::new(full_data_model, state_to_bind).unwrap();
        let text_input_cb_id = window.add_callback(ptr, DefaultCallback(Self::update_state_on_text_input));
        let vk_down_cb_id = window.add_callback(ptr, DefaultCallback(Self::update_state_on_vk_down));

        Self {
            text_input_cb_id,
			vk_down_cb_id,
            mrkr: PhantomData,
        }
    }

    pub fn dom(self, state_to_render: &PadState) -> Dom<T> {
        let term_str = create_terminal_string(&state_to_render.terminal);

        let categorised = cansi::categorise_text(&term_str);

        let mut container = Dom::div()
            .with_class("terminal")
            // .with_callback(On::TextInput, Callback(on_text_input))
            // .with_callback(On::VirtualKeyDown, Callback(on_vk_keydown))
            .with_tab_index(TabIndex::Auto); // make focusable
        container.add_default_callback_id(On::TextInput, self.text_input_cb_id);
		container.add_default_callback_id(On::VirtualKeyDown, self.vk_down_cb_id);

        for line in cansi::line_iter(&categorised) {
            let mut line_div = Dom::div().with_class("terminal-line");
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
    let s = String::from_utf8_lossy(cat_slice.text_as_bytes);

    Dom::label(s).with_class("terminal-text").with_css_override(
        PROPERTY_STR,
        StyleTextColor(widgets::colour::map(&cat_slice.fg_colour)).into(),
    )
}
