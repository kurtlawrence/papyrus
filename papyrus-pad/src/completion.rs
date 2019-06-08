use super::*;
use papyrus::complete;
use papyrus::prelude::*;
use std::sync::{Arc, Mutex};

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

    fn mut_op<F: FnOnce(&mut CompletionData)>(&mut self, func: F) {
        // try to avoid locking if possible
        if let Some(m) = Arc::get_mut(&mut self.data) {
            func(m.get_mut().unwrap())
        } else {
            func(&mut self.data.lock().unwrap())
        }
    }

    /// Creates a completion task to be run on another thread.
    pub fn complete<T>(&mut self, line: &str, limit: Option<usize>) -> Task<T> {
        self.mut_op(|data| {
            data.line.clear();
            data.line.push_str(line);
            data.limit = limit;
        });

        Task::new(&self.data, complete_task)
    }

    // Should be prefaced with reset call
    pub fn build_completers<D>(&mut self, repl_data: &ReplData<D>) {
        self.mut_op(|data| data.completers = Completers::build(repl_data));
    }

    // resets the internal completions state
    pub fn reset(&mut self) {
        self.mut_op(|data| {
            data.line.clear();
            data.limit = None;
            data.completions.clear();
        })
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
    pub fn dom<T>(
        completions: Vec<String>,
    ) -> Dom<T> {
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
