use super::*;
use papyrus::complete;
use papyrus::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

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
pub struct CompletionPromptState {
    /// Completion data.
    data: Arc<Mutex<Completers>>,

    /// A completion item waiting to complete.
    line_msg: Arc<ThreadedOption<CompletionInput>>,

    /// Completed items, waiting to be updated into `last_completions`.
    completions_msg: Arc<ThreadedOption<Vec<String>>>,

    /// The last iteration of completion items.
    pub last_completions: Vec<String>,
}

impl CompletionPromptState {
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
    pub fn dom<T>(completions: Vec<String>) -> Dom<T> {
        let container = completions
            .into_iter()
            .map(|x| Dom::label(x))
            .collect::<Dom<T>>()
            .with_id("completion-prompt")
            .with_tab_index(TabIndex::Auto); // make focusable

        container
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
