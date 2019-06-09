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

pub struct CompletionPromptState {
    data: Arc<Mutex<Completers>>,

    line_msg: Arc<ThreadedOption<CompletionInput>>,

    completions: Arc<Mutex<Vec<String>>>,
}

impl CompletionPromptState {
    pub fn initialise<D>(repl_data: &ReplData<D>) -> Self {
        let data = Arc::new(Mutex::new(Completers::build(repl_data)));

        let line_msg = Arc::new(ThreadedOption::empty());

        let completions_var = Arc::new(Mutex::new(Vec::new()));

        let ct_data = Arc::clone(&data);
        let ct_line_msg = Arc::clone(&line_msg);
        let ct_completions = Arc::clone(&completions_var);

        let ret = Self {
            data,
            line_msg,
            completions: completions_var,
        };

        std::thread::spawn(move || {
            // completions run on a separate thread
            // tried using a azul::Task but spinning up a thread for each char input
            // was lagging, so instead we run on one other thread and use message passing

            let line_msg = ct_line_msg;
            let data = ct_data;
            let completions_option = ct_completions;

            loop {
                if let Some(input) = line_msg.take() {
                    let completers = data.lock().unwrap();

                    let completed = completions(&completers, &input.line, input.limit);

                    *completions_option.lock().unwrap() = completed;
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(20)); // only check every so often
                }
            }
        });

        ret
    }

    pub fn to_complete(&mut self, line: String, limit: Option<usize>) {
        self.line_msg.put(CompletionInput { line, limit });
    }

    pub fn completions(&self) -> Vec<String> {
        self.completions.lock().unwrap().clone()
    }

    pub fn build_completers<D>(&mut self, repl_data: &ReplData<D>) {
        *self.data.lock().unwrap() = Completers::build(repl_data);
    }
}

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

impl CompletionPrompt {
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
