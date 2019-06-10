use super::*;
use azul::{
    app::AppStateNoData,
    callbacks::DefaultCallback,
    css::{CssProperty, LayoutHeight},
};
use papyrus::complete;
use papyrus::prelude::*;
use std::cmp::min;
use std::marker::PhantomData;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

const FOCUS_LMD: EventFilter = EventFilter::Focus(FocusEventFilter::LeftMouseDown);
const WINDOW_LMD: EventFilter = EventFilter::Window(WindowEventFilter::LeftMouseDown);
const FOCUS_VKDOWN: EventFilter = EventFilter::Focus(FocusEventFilter::VirtualKeyDown);

const ITEM_HEIGHT: f32 = 20.0;

struct ThreadedOption<T> {
    data: Mutex<Option<T>>,
    changed: AtomicBool,
}

impl<T> ThreadedOption<T> {
    /// Constructs new message with no value.
    fn empty() -> Self {
        Self {
            data: Mutex::new(None),
            changed: AtomicBool::default(),
        }
    }

    /// Fast check if a message is waiting.
    fn has(&self) -> bool {
        self.changed.load(Ordering::SeqCst)
    }

    /// Place a value, overwriting any previous value.
    fn put(&self, value: T) {
        *self.data.lock().unwrap() = Some(value);
        self.changed.store(true, Ordering::SeqCst);
    }

    /// Take the value if one exists.
    fn take(&self) -> Option<T> {
        if self.has() {
            self.changed.store(false, Ordering::SeqCst);
            self.data.lock().unwrap().take()
        } else {
            None
        }
    }
}

struct CompletionInput {
    pub line: String,
    pub limit: Option<usize>,
}

/// Holds state of a `CompletionPrompt`.
pub struct CompletionPromptState<T> {
    /// Completion data.
    data: Arc<Mutex<Completers>>,

    /// A completion item waiting to complete.
    line_msg: Arc<ThreadedOption<CompletionInput>>,

    /// Completed items, waiting to be updated into `last_completions`.
    completions_msg: Arc<ThreadedOption<Vec<String>>>,

    /// The last iteration of completion items.
    pub last_completions: Vec<String>,

    // Rendering info
    pub kb_focus: usize,

    mrkr: PhantomData<T>,
}

impl<T> CompletionPromptState<T> {
    /// Build completion data and spin off a completing thread.
    pub fn initialise<D>(repl_data: &ReplData<D>) -> Self {
        let data = Arc::new(Mutex::new(Completers::build(repl_data)));

        let line_msg = Arc::new(ThreadedOption::empty());

        let completions_msg = Arc::new(ThreadedOption::empty());

        let last_completions = Vec::new();

        let ct_data = Arc::clone(&data);
        let ct_line_msg = Arc::clone(&line_msg);
        let ct_completions_msg = Arc::clone(&completions_msg);

        let ret = Self {
            data,
            line_msg,
            completions_msg,
            last_completions,
            kb_focus: 0,
            mrkr: PhantomData,
        };

        std::thread::spawn(move || {
            // completions run on a separate thread
            // tried using a azul::Task but spinning up a thread for each char input
            // was lagging, so instead we run on one other thread and use message passing

            let line_msg = ct_line_msg;
            let data = ct_data;
            let completions_msg = ct_completions_msg;

            loop {
                if let Some(input) = line_msg.take() {
                    let completers = data.lock().unwrap();

                    let completed = completions(&completers, &input.line, input.limit);

                    completions_msg.put(completed);
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(20)); // only check every so often
                }
            }
        });

        ret
    }

    /// Will render if `last_completions` is not empty.
    pub fn will_render(&self) -> bool {
        !self.last_completions.is_empty()
    }

    /// Send a line to be completed, with a limit of number of completions.
    pub fn to_complete(&mut self, line: String, limit: Option<usize>) {
        self.line_msg.put(CompletionInput { line, limit });
    }

    /// Read if there are completed items waiting to be set into `last_completions`.
    /// Returns true if therer were items.
    pub fn update(&mut self) -> bool {
        if let Some(completions) = self.completions_msg.take() {
            self.last_completions = completions;
            true
        } else {
            false
        }
    }

    /// Update completion data with repl data state.
    pub fn build_completers<D>(&mut self, repl_data: &ReplData<D>) {
        *self.data.lock().unwrap() = Completers::build(repl_data);
    }

    pub fn on_focus_vk_down(
        &mut self,
        app: &mut AppStateNoData<T>,
        event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        use AcceleratorKey::*;
        use VirtualKeyCode::*;

        let kb = app.windows[event.window_id].get_keyboard_state();

        kb_seq(kb, &[Key(Up)], || {
            self.kb_focus = self.kb_focus.saturating_sub(1)
        })
        .or_else(|| {
            kb_seq(kb, &[Key(Down)], || {
                self.kb_focus = min(
                    self.kb_focus + 1,
                    self.last_completions.len().saturating_sub(1),
                )
            })
        })
    }

    fn on_focus_left_mouse_down(
        &mut self,
        app: &mut AppStateNoData<T>,
        event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        DontRedraw
    }

    fn on_window_left_mouse_down(
        &mut self,
        app: &mut AppStateNoData<T>,
        event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        DontRedraw
    }
}

impl<T: 'static> CompletionPromptState<T> {
    cb!(priv_on_focus_vk_down, on_focus_vk_down);
    cb!(priv_on_focus_left_mouse_down, on_focus_left_mouse_down);
    cb!(priv_on_window_left_mouse_down, on_window_left_mouse_down);
}

struct Completers {
    cmds_tree: complete::cmdr::TreeCompleter,
    mods: complete::modules::ModulesCompleter,
    code: complete::code::CodeCompleter,
}

impl Completers {
    fn build<D>(repl_data: &ReplData<D>) -> Self {
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

/// Draw a completion prompt.
pub struct CompletionPrompt;

impl CompletionPrompt {
    /// Draw a completion prompt with the completions.
    /// Will return `None` if `last_completions` is empty.
    pub fn dom<T: 'static>(
        state: &AppValue<CompletionPromptState<T>>,
        info: &mut LayoutInfo<T>,
    ) -> Option<Dom<T>> {
        if state.last_completions.is_empty() {
            None
        } else {
            let ptr = StackCheckedPointer::new(state);

            let focus_vk_down_cb_id = info.window.add_callback(
                ptr.clone(),
                DefaultCallback(CompletionPromptState::priv_on_focus_vk_down),
            );
            let focus_left_mouse_down_cb_id = info.window.add_callback(
                ptr.clone(),
                DefaultCallback(CompletionPromptState::priv_on_focus_left_mouse_down),
            );
            let window_left_mouse_down_cb_id = info.window.add_callback(
                ptr.clone(),
                DefaultCallback(CompletionPromptState::priv_on_window_left_mouse_down),
            );

            let mut container = state
                .last_completions
                .iter()
                .enumerate()
                .map(|(idx, x)| {
                    let mut item = Dom::label(x.to_owned())
                        .with_class("completion-prompt-item")
                        .with_css_override(
                            "height",
                            CssProperty::Height(LayoutHeight::px(ITEM_HEIGHT)),
                        );

                    if idx == state.kb_focus {
                        item.add_class("completion-prompt-item-kb");
                    }

                    item
                })
                .collect::<Dom<T>>()
                .with_class("completion-prompt")
                .with_tab_index(TabIndex::Auto); // make focusable

            container.add_default_callback_id(FOCUS_VKDOWN, focus_vk_down_cb_id);
            container.add_default_callback_id(FOCUS_LMD, focus_left_mouse_down_cb_id);
            container.add_default_callback_id(WINDOW_LMD, window_left_mouse_down_cb_id);

            Some(container)
        }
    }
}

fn completions(completer: &Completers, line: &str, limit: Option<usize>) -> Vec<String> {
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

    if !line.starts_with('.') {
        // don't do code completion on command
        v.extend(
            completer
                .code
                .complete(line, Some(limit.saturating_sub(v.len())))
                .into_iter()
                .map(|x| x.matchstr),
        );
    }

    v
}
