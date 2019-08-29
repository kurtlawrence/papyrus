use super::*;
use azul::{
    app::AppStateNoData,
    callbacks::DefaultCallback,
    css::{CssProperty, LayoutHeight},
};
use papyrus::complete::{
    cmdr::TreeCompleter,
    code::{CodeCache, CodeCompleter},
    modules::ModulesCompleter,
};
use papyrus::prelude::*;
use papyrus::racer::MatchType;
use std::borrow::Cow;
use std::cmp::min;
use std::marker::PhantomData;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

const FOCUS_VKDOWN: EventFilter = EventFilter::Focus(FocusEventFilter::VirtualKeyDown);

const ITEM_HEIGHT: f32 = 20.0;

/// Holds state of a `CompletionPrompt`.
pub struct CompletionPromptState<T> {
    /// Completion data.
    data: Arc<Mutex<Completers>>,

    /// A completion item waiting to complete.
    line_msg: Arc<ThreadedOption<CompletionInput>>,

    /// Completed items, waiting to be updated into `last_completions`.
    completions_msg: Arc<ThreadedOption<Completions>>,

    /// The last iteration of completion items.
    last_completions: Completions,

    // Rendering info
    pub kb_focus: usize,
    pub last_mouse_hovered: Option<usize>,

    mrkr: PhantomData<T>,
}

impl<T> CompletionPromptState<T> {
    /// Build completion data and spin off a completing thread.
    pub fn initialise<D>(repl_data: &ReplData<D>) -> Self {
        let data = Arc::new(Mutex::new(Completers::build(repl_data)));

        let line_msg = Arc::new(ThreadedOption::empty());

        let completions_msg = Arc::new(ThreadedOption::empty());

        let last_completions = Completions::new();

        let ct_data = Arc::clone(&data);
        let ct_line_msg = Arc::clone(&line_msg);
        let ct_completions_msg = Arc::clone(&completions_msg);

        let ret = Self {
            data,
            line_msg,
            completions_msg,
            last_completions,
            kb_focus: 0,
            last_mouse_hovered: None,
            mrkr: PhantomData,
        };

        std::thread::spawn(move || {
            // completions run on a separate thread
            // tried using a azul::Task but spinning up a thread for each char input
            // was lagging, so instead we run on one other thread and use message passing

            let line_msg = ct_line_msg;
            let data = ct_data;
            let completions_msg = ct_completions_msg;
            let code_cache = CodeCache::new();

            loop {
                if let Some(input) = line_msg.take() {
                    let completers = data.lock().unwrap();

                    let CompletionInput {
                        input_buffer,
                        input_buffer_line,
                        limit,
                    } = input;

                    let items = completions(
                        &completers,
                        &input_buffer,
                        &input_buffer_line,
                        limit,
                        &code_cache,
                    );

                    let completions = Completions {
                        input_buffer_line,
                        items,
                    };

                    completions_msg.put(completions);
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(20)); // only check every so often
                }
            }
        });

        ret
    }

    /// Will render if `last_completions` is not empty.
    pub fn will_render(&self) -> bool {
        !self.last_completions.items.is_empty()
    }

    /// Uses the completion at index to complete the input buffer line.
    ///
    /// The input buffer line completed is the version that was used to seed
    /// the completion.
    /// If an item exists at the index, this function will consume _all_ the
    /// completions. Otherwise it will be left alone.
    pub fn complete_input_buffer_line(&mut self, index: usize) -> Option<String> {
        if index < self.last_completions.items.len() {
            let Completions {
                mut input_buffer_line,
                mut items,
                ..
            } = std::mem::replace(&mut self.last_completions, Completions::new());

            let item = items.remove(index);

            input_buffer_line.replace_range(item.word_start.., &item.completion);

            if let Some(ch) = item.suffix {
                input_buffer_line.push(ch);
            }

            Some(input_buffer_line)
        } else {
            None
        }
    }

    /// Send a line to be completed, with a limit of number of completions.
    pub fn to_complete(
        &mut self,
        input_buffer: String,
        input_buffer_line: String,
        limit: Option<usize>,
    ) {
        self.line_msg.put(CompletionInput {
            input_buffer,
            input_buffer_line,
            limit,
        });
    }

    /// Read if there are completed items waiting to be set into `last_completions`.
    /// Returns true if there were items.
    pub fn update(&mut self) -> bool {
        if let Some(completions) = self.completions_msg.take() {
            self.set_last(completions);
            true
        } else {
            false
        }
    }

    /// Update completion data with repl data state.
    pub fn build_completers<D>(&mut self, repl_data: &ReplData<D>) {
        *self.data.lock().unwrap() = Completers::build(repl_data);
    }

    /// Use this as it resets the focusing indices
    fn set_last(&mut self, completions: Completions) {
        self.last_completions = completions;
        self.kb_focus = 0;
        self.last_mouse_hovered = None;
    }

    /// Removes latest completions and reset focusing indices.
    pub fn clear(&mut self) {
        self.set_last(Completions::new());
    }

    fn on_focus_vk_down_inner(info: DefaultCallbackInfo<T, Self>) -> UpdateScreen {
        let kb = info.get_keyboard_state().clone();
        info.data.on_focus_vk_down(&kb)
    }

    pub fn on_focus_vk_down(&mut self, kb: &KeyboardState) -> UpdateScreen {
        use AcceleratorKey::*;
        use VirtualKeyCode::*;

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
        .or_else(|| kb_seq(kb, &[Key(Escape)], || self.clear()))
    }

    fn on_mouse_enter(info: DefaultCallbackInfo<T, Self>) -> UpdateScreen {
        info.data.last_mouse_hovered = info.get_index_in_parent(&info.hit_dom_node).map(|x| x.0);
        DontRedraw
    }

    fn on_mouse_leave(info: DefaultCallbackInfo<T, Self>) -> UpdateScreen {
        let (idx, _) = info.get_index_in_parent(&info.hit_dom_node)?;

        info.data.last_mouse_hovered = None;

        DontRedraw
    }
}

impl<T: 'static> CompletionPromptState<T> {
    cb!(priv_on_focus_vk_down, on_focus_vk_down_inner);
    cb!(priv_on_mouse_enter, on_mouse_enter);
    cb!(priv_on_mouse_leave, on_mouse_leave);
}

/// Draw a completion prompt.
pub struct CompletionPrompt;

impl CompletionPrompt {
    /// Draw a completion prompt with the completions.
    /// Will return `None` if `last_completions` is empty.
    /// `top` and `left` are relative to the container they are in.
    pub fn dom<T: 'static>(
        state: &AppValue<CompletionPromptState<T>>,
        info: &mut LayoutInfo<T>,
        top: f32,
        left: f32,
    ) -> Option<Dom<T>> {
        if state.last_completions.is_empty() {
            None
        } else {
            let ptr = StackCheckedPointer::new(state);

            let focus_vkdown_cb_id = info
                .window
                .add_default_callback(CompletionPromptState::priv_on_focus_vk_down, ptr.clone());
            let menter_cb_id = info
                .window
                .add_default_callback(CompletionPromptState::priv_on_mouse_enter, ptr.clone());
            let mleave_cb_id = info
                .window
                .add_default_callback(CompletionPromptState::priv_on_mouse_leave, ptr.clone());

            let mut prompt = state
                .last_completions
                .items
                .iter()
                .enumerate()
                .map(|(idx, x)| {
                    let mut item = Dom::div()
                        .with_class("completion-prompt-item")
                        .with_css_override("height", LayoutHeight::px(ITEM_HEIGHT).into())
                        .with_tab_index(TabIndex::Auto)
                        .with_child(Dom::label(x.completion.to_owned()))
                        .with_child(
                            Dom::label(x.completion_type.dom_string())
                                .with_class("completion-prompt-item-type"),
                        );

                    item.add_default_callback_id(On::MouseEnter, menter_cb_id);
                    item.add_default_callback_id(On::MouseLeave, mleave_cb_id);

                    if idx == state.kb_focus {
                        item.add_class("completion-prompt-item-kb");
                    }

                    item
                })
                .collect::<Dom<T>>()
                .with_class("completion-prompt")
                .with_css_override("top", LayoutTop::px(top).into())
                .with_css_override("left", LayoutLeft::px(left).into())
                .with_tab_index(TabIndex::Auto); // make focusable

            prompt.add_default_callback_id(FOCUS_VKDOWN, focus_vkdown_cb_id);

            // Add a context and documentation panel beside other one.
            if let Some(item) = state.last_completions.items.get(state.kb_focus) {
                prompt.add_child(
                    Dom::div()
                        .with_class("completion-prompt-info")
                        .with_css_override(
                            "top",
                            LayoutTop::px(state.kb_focus as f32 * ITEM_HEIGHT).into(),
                        )
                        .with_child(Dom::label(item.contextstr.to_string()))
                        .with_child(Dom::label(
                            item.docs.lines().next().unwrap_or("").to_owned(),
                        )),
                );
            }

            Some(prompt)
        }
    }
}

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
    pub input_buffer: String,
    pub input_buffer_line: String,
    pub limit: Option<usize>,
}

struct Completers {
    cmds_tree: TreeCompleter,
    mods: ModulesCompleter,
    code: CodeCompleter,
}

impl Completers {
    fn build<D>(repl_data: &ReplData<D>) -> Self {
        let cmds_tree = TreeCompleter::build(&repl_data.cmdtree);

        let mods = ModulesCompleter::build(&repl_data.cmdtree, &repl_data.mods_map());

        let code = CodeCompleter::build(repl_data);

        Self {
            cmds_tree,
            mods,
            code,
        }
    }
}

struct Completions {
    input_buffer_line: String,
    items: Vec<Completion>,
}

impl Completions {
    /// No allocation when empty.
    pub fn new() -> Self {
        Self {
            input_buffer_line: String::new(),
            items: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

struct Completion {
    completion: String,
    completion_type: CompletionType,
    word_start: usize,
    suffix: Option<char>,
    contextstr: Cow<'static, str>,
    docs: Cow<'static, str>,
}

#[derive(Copy, Clone)]
enum CompletionType {
    CmdAction,
    CmdClass,
    Enum,
    EnumVariant,
    Fn,
    Macro,
    Mod,
    Primitive,
    Struct,
    Trait,
    TreeMods,
    Type,
    Unknown,
    Variable,
}

impl CompletionType {
    fn dom_string(&self) -> DomString {
        match self {
            CompletionType::CmdAction => "cmd-action",
            CompletionType::CmdClass => "cmd-class",
            CompletionType::Enum => "enum",
            CompletionType::EnumVariant => "variant",
            CompletionType::Fn => "fn",
            CompletionType::Macro => "macro",
            CompletionType::Mod => "mod",
            CompletionType::Primitive => "prim",
            CompletionType::Struct => "struct",
            CompletionType::Trait => "trait",
            CompletionType::TreeMods => "mod-path",
            CompletionType::Type => "type",
            CompletionType::Unknown => "?",
            CompletionType::Variable => "var",
        }
        .into()
    }
}

fn completions(
    completer: &Completers,
    input_buffer: &str,
    input_buffer_line: &str,
    limit: Option<usize>,
    code_cache: &CodeCache,
) -> Vec<Completion> {
    let limit = limit.unwrap_or(std::usize::MAX);

    let mut v = Vec::<Completion>::new();

    {
        let word_start = TreeCompleter::word_break(input_buffer_line);
        let suffix = Some(' ');

        v.extend({
            completer
                .cmds_tree
                .complete(input_buffer_line)
                .take(limit)
                .map(|x| {
                    use papyrus::cmdtree::ItemType;

                    let (comp, info) = x;

                    let completion_type = match info.itemtype {
                        ItemType::Action => CompletionType::CmdAction,
                        ItemType::Class => CompletionType::CmdClass,
                    };

                    Completion {
                        completion: comp.to_owned(),
                        completion_type,
                        word_start,
                        suffix,
                        contextstr: info.help_msg.inner_cow.clone(),
                        docs: Cow::Borrowed(""),
                    }
                })
        });
    }

    if let Some(iter) = completer.mods.complete(input_buffer_line) {
        let word_start = ModulesCompleter::word_break(input_buffer_line);
        let completion_type = CompletionType::TreeMods;
        let suffix = Some(' ');

        v.extend(
            iter.take(limit.saturating_sub(v.len()))
                .map(|completion| Completion {
                    completion,
                    completion_type,
                    word_start,
                    suffix,
                    contextstr: Cow::Borrowed(""),
                    docs: Cow::Borrowed(""),
                }),
        );
    }

    if !input_buffer_line.starts_with('.') {
        // don't do code completion on command
        let word_start = CodeCompleter::word_break(input_buffer_line);
        let suffix = None;

        v.extend(
            completer
                .code
                .complete(
                    input_buffer,
                    Some(limit.saturating_sub(v.len())),
                    code_cache,
                )
                .into_iter()
                .map(|x| Completion {
                    completion: x.matchstr,
                    completion_type: x.mtype.into(),
                    word_start,
                    suffix,
                    contextstr: Cow::Owned(x.contextstr),
                    docs: Cow::Owned(x.docs),
                }),
        );
    }

    v
}

impl From<MatchType> for CompletionType {
    fn from(mtype: MatchType) -> Self {
        match mtype {
            MatchType::Builtin(_) => CompletionType::Primitive,
            MatchType::Enum(_) => CompletionType::Enum,
            MatchType::EnumVariant(_) => CompletionType::EnumVariant,
            MatchType::Function => CompletionType::Fn,
            MatchType::Let(_) => CompletionType::Variable,
            MatchType::Macro => CompletionType::Macro,
            MatchType::Module => CompletionType::Mod,
            MatchType::Struct(_) => CompletionType::Struct,
            MatchType::Trait => CompletionType::Trait,
            MatchType::Type => CompletionType::Type,
            x => {
                eprintln!("need to handle MatchType");
                dbg!(x);
                CompletionType::Unknown
            }
        }
    }
}
