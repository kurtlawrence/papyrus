use super::*;
use azul::app::AppStateNoData;
use azul::callbacks::{DefaultCallback, DefaultCallbackId};
use azul::prelude::*;
use azul::window::FakeWindow;
use papyrus::complete;
use papyrus::prelude::*;
use std::borrow::BorrowMut;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

type KickOffEvalDaemon = bool;
type HandleCb = (UpdateScreen, KickOffEvalDaemon);

pub struct CompletionPromptState {
    pub data: Arc<Mutex<CompletionData>>,
}

impl CompletionPromptState {
    pub fn new<D>(repl_data: &ReplData<D>) -> Self {
        Self {
            data: Arc::new(Mutex::new(CompletionData {
                completers: Completers::build(repl_data),
                line: String::new(),
                limit: None,
                completions: Vec::new(),
            })),
        }
    }

    /// Creates a completion task to be run on another thread.
    pub fn complete<T>(&self, line: String, limit: Option<usize>) -> Task<T> {
        {
            let mut lock = self.data.lock().unwrap();

            lock.line = line;
            lock.limit = limit;
        }

        Task::new(&self.data, complete_task)
    }
}

fn complete_task(data: Arc<Mutex<CompletionData>>, _: DropCheck) {
    let mut lock = data.lock().unwrap();

    let completions = completions(&lock.completers, &lock.line, lock.limit);

    lock.completions = completions;
}

pub struct CompletionData {
    pub completers: Completers,

    pub line: String,
    pub limit: Option<usize>,

    pub completions: Vec<String>,
}

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

pub fn completions(completer: &Completers, line: &str, limit: Option<usize>) -> Vec<String> {
    let limit = limit.unwrap_or(std::usize::MAX);

    let mut v = Vec::<String>::new();

    v.extend(
        completer
            .cmds_tree
            .complete(line)
            .map(|x| x.to_string())
            .take(limit),
    );

    if let Some(i) = completer.mods.complete(line) {
        v.extend(i.take(limit.saturating_sub(v.len())));
    }

    v.extend(
        completer
            .code
            .complete(line, Some(limit.saturating_sub(v.len())))
            .into_iter()
            .map(|x| x.matchstr),
    );

    v
}
