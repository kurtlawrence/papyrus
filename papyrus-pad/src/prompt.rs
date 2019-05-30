use super::*;
use azul::app::AppStateNoData;
use azul::callbacks::{DefaultCallback, DefaultCallbackId};
use azul::prelude::*;
use azul::window::FakeWindow;
use papyrus::prelude::*;
use std::borrow::BorrowMut;
use std::marker::PhantomData;
use papyrus::complete;

type KickOffEvalDaemon = bool;
type HandleCb = (UpdateScreen, KickOffEvalDaemon);

impl<T, D> PadState<T, D> {
    // fn handle_vk(&mut self, vk: VirtualKeyCode) -> HandleCb {
    //     match vk {
    //         VirtualKeyCode::Back => self.handle_input('\x08'), // backspace character
    //         VirtualKeyCode::Tab => self.handle_input('\t'),
    //         VirtualKeyCode::Return => self.handle_input('\n'),
    //         _ => (DontRedraw, false),
    //     }
    // }
}

impl<T, D> PadState<T, D> {
    // fn update_state_on_vk_down(
    //     &mut self,
    //     app_state: &mut AppStateNoData<T>,
    //     window_event: &mut CallbackInfo<T>,
    // ) -> UpdateScreen {
    //     let (update_screen, kickoff) = self.handle_vk(
    //         app_state.windows[window_event.window_id]
    //             .get_keyboard_state()
    //             .latest_virtual_keycode?,
    //     );

    //     if kickoff {
    //         kickoff_daemon(app_state, self.eval_daemon_id);
    //     }

    //     update_screen
    // }
}

// pub struct CompletionPrompt<T, D> {
//     mrkr: PhantomData<T>,
//     mrkr_data: PhantomData<D>,
// }

pub struct Completers {
    pub cmds_tree: complete::cmdr::TreeCompleter,
    pub mods: complete::modules::ModulesCompleter,
    pub code: complete::code::CodeCompleter,
}

impl Completers {
	pub fn build<D>(repl_data: &ReplData<D>) -> Self {
    let cmds_tree = complete::cmdr::TreeCompleter::build(&repl_data.cmdtree);

    let mods =
        complete::modules::ModulesCompleter::build(&repl_data.cmdtree, &repl_data.file_map());

    let code = complete::code::CodeCompleter::build(repl_data);

    Self {
        cmds_tree,
        mods,
        code,
    }
}
}



pub struct CompletionPrompt;

// impl<T, D> CompletionPrompt<T, D>
impl CompletionPrompt {
    pub fn dom<T, D>(
        pad_state: &AppValue<PadState<T, D>>,
        window: &mut FakeWindow<T>,
        completions: Vec<String>,
    ) -> Dom<T> {
        // let ptr = StackCheckedPointer::new(state);

        // let vk_down_cb_id =
        // window.add_callback(ptr.clone(), DefaultCallback(Self::update_state_on_vk_down));

        // container.add_default_callback_id(On::TextInput, text_input_cb_id);
        // container.add_default_callback_id(On::VirtualKeyDown, vk_down_cb_id);

        let mut container = completions
            .into_iter()
            .map(|x| Dom::label(x))
            .collect::<Dom<T>>()
            .with_id("completion-prompt")
            .with_tab_index(TabIndex::Auto); // make focusable

        container
    }

    // cb!(PadState, update_state_on_vk_down);
}

pub fn completions(completer: &Completers, line: &str) -> Vec<String> {
    let mut v = Vec::<String>::new();

    v.extend(completer.cmds_tree.complete(line).map(|x| x.to_string()));

    if let Some(i) = completer.mods.complete(line) {
        v.extend(i);
    }

    v.extend(
        completer
            .code
            .complete(line)
            .into_iter()
            .map(|x| x.matchstr),
    );

    v
}
