use super::*;
use crate::repl::{Editing, EditingIndex, ReplData};
use cmdtree::{BuildError, Builder, BuilderChain, Commander};
use std::io::Write;
use std::path::{Path, PathBuf};

pub use cmdtree::Builder as CommandBuilder;

/// The action to take. Passes through a mutable reference to the `ReplData`.
pub type ReplDataAction<D> = Box<dyn Fn(&mut ReplData<D>, &mut dyn Write) -> String>;

/// The action to take. Passes through a mutable reference to the data `D`.
pub type AppDataAction<D> = Box<dyn Fn(&mut D, &mut dyn Write) -> String>;

/// The result of a [`cmdtree action`].
/// This result is handed in the repl's evaluating stage, and can alter `ReplData` or the data `D`.
///
/// [`cmdtree action`]: cmdtree::Action
pub enum CommandResult<D> {
    /// Flag to begin a mutating block.
    BeginMutBlock,
    /// Flag to alter a previous statement, item, or crate.
    EditAlter(EditingIndex),
    /// Replace a previous statement, item, or crate with value.
    EditReplace(EditingIndex, String),
    /// Switch to a module.
    SwitchModule(PathBuf),
    /// Take an action on the `ReplData`.
    ActionOnReplData(ReplDataAction<D>),
    /// Take an action on `Data`.
    ActionOnAppData(AppDataAction<D>),
    /// A blank variant with no action.
    Empty,
}

impl<D> CommandResult<D> {
    /// Convenience function boxing an action on app data.
    pub fn app_data_fn<F: 'static + Fn(&mut D, &mut dyn Write) -> String>(func: F) -> Self {
        CommandResult::ActionOnAppData(Box::new(func))
    }

    /// Convenience function boxing an action on repl data.
    pub fn repl_data_fn<F: 'static + Fn(&mut ReplData<D>, &mut dyn Write) -> String>(
        func: F,
    ) -> Self {
        CommandResult::ActionOnReplData(Box::new(func))
    }
}

impl<D> ReplData<D> {
    /// Uses the given `Builder` as the root of the command tree.
    ///
    /// An error will be returned if any command already exists.
    pub fn with_cmdtree_builder(
        &mut self,
        builder: Builder<CommandResult<D>>,
    ) -> Result<&mut Self, BuildError> {
        self.cmdtree = papyrus_cmdr(builder)?;
        Ok(self)
    }
}

fn papyrus_cmdr<D>(
    builder: Builder<CommandResult<D>>,
) -> Result<Commander<CommandResult<D>>, BuildError> {
    builder
        .root()
        .add_action("mut", "Begin a mutable block of code", |_, _| {
            CommandResult::BeginMutBlock
        })
        .begin_class("edit", "Edit previous input")
        .begin_class("stmt", "Edit previous statements")
        .add_action(
            "alter",
            "Alter statement contents. args: stmt-number",
            |wtr, args| edit_alter_priv(args, wtr, Editing::Stmt),
        )
        .add_action(
            "replace",
            "Replace statement contents. args: stmt-number value",
            |wtr, args| edit_replace_priv(args, wtr, Editing::Stmt),
        )
        .end_class()
        .end_class()
        .begin_class("mod", "Handle modules")
        .add_action(
            "switch",
            "Switch to a module, creating one if necessary. switch path/to/module",
            |wtr, args| switch_module_priv(args, wtr),
        )
        .end_class()
        .into_commander()
}

fn switch_module_priv<D, W: Write>(args: &[&str], mut wtr: W) -> CommandResult<D> {
    if let Some(path) = args.get(0) {
        if let Some(path) = make_path(path) {
            CommandResult::SwitchModule(path)
        } else {
            writeln!(wtr, "failed to parse {} into a valid module path", path).unwrap();
            CommandResult::Empty
        }
    } else {
        writeln!(wtr, "switch expects a path to module argument").unwrap();
        CommandResult::Empty
    }
}

fn make_all_parents(path: &Path) -> Vec<PathBuf> {
    let components: Vec<_> = path.iter().collect();

    (1..components.len())
        .into_iter()
        .map(|idx| components[0..idx].iter().collect::<PathBuf>())
        .collect()
}

fn make_path(path: &str) -> Option<PathBuf> {
    let path = path.trim();

    let path = path.replace(".rs", "").replace("mod", "").replace("-", "_");

    if path == "lib" {
        return Some(PathBuf::from("lib"));
    }

    let x: &[_] = &['/', '\\'];
    let path = path.trim_matches(x); // remove starting or trailing slashes

    if path.is_empty() {
        return None;
    }

    Some(PathBuf::from(path))
}

fn edit_alter_priv<D, W: Write>(args: &[&str], mut wtr: W, t: Editing) -> CommandResult<D> {
    if let Some(idx) = args.get(0) {
        match parse_idx(idx, t) {
            Ok(ei) => CommandResult::EditAlter(ei),
            Err(e) => {
                writeln!(wtr, "failed parsing {} as number: {}", idx, e).ok();
                CommandResult::Empty
            }
        }
    } else {
        writeln!(wtr, "alter expects an index number").ok();
        CommandResult::Empty
    }
}

fn edit_replace_priv<D, W: Write>(args: &[&str], mut wtr: W, t: Editing) -> CommandResult<D> {
    if let Some(idx) = args.get(0) {
        match parse_idx(idx, t) {
            Ok(ei) => {
                CommandResult::EditReplace(ei, args[1..].iter().map(|x| *x).collect::<String>())
            }
            Err(e) => {
                writeln!(wtr, "failed parsing {} as number: {}", idx, e).ok();
                CommandResult::Empty
            }
        }
    } else {
        writeln!(wtr, "replace expects an index number").ok();
        CommandResult::Empty
    }
}

fn parse_idx(s: &str, editing: Editing) -> Result<EditingIndex, String> {
    s.parse()
        .map_err(|e| format!("{}", e))
        .map(|index| EditingIndex { editing, index })
}

pub(crate) fn edit_alter<D>(data: &mut ReplData<D>, ei: EditingIndex) -> &'static str {
    let src = data.current_src();

    let len = match ei.editing {
        Editing::Stmt => src.stmts.len(),
        Editing::Item => src.items.len(),
        Editing::Crate => src.crates.len(),
    };

    if ei.index >= len {
        "index is outside of range"
    } else {
        data.editing = Some(ei);
        ""
    }
}

pub(crate) fn switch_module<D>(data: &mut ReplData<D>, path: &Path) -> &'static str {
    let mut all = make_all_parents(path);
    all.push(path.to_path_buf());

    for x in all {
        if !data.mods_map.contains_key(&x) {
            data.mods_map.insert(x, pfh::SourceCode::new());
        }
    }

    data.current_mod = path.to_path_buf();

    ""
}

#[test]
fn make_path_test() {
    assert_eq!(make_path("   "), None);

    assert_eq!(make_path("lib"), Some(PathBuf::from("lib")));
    assert_eq!(make_path("lib.rs"), Some(PathBuf::from("lib")));

    assert_eq!(make_path("test"), Some(PathBuf::from("test")));
    assert_eq!(make_path("test/inner"), Some(PathBuf::from("test/inner")));
    assert_eq!(make_path("inner/test"), Some(PathBuf::from("inner/test")));

    assert_eq!(make_path("//"), None);

    assert_eq!(make_path("\\hello\\"), Some(PathBuf::from("hello")));
}

#[test]
fn make_all_parents_test() {
    // only handle parents
    assert_eq!(make_all_parents(Path::new("")), Vec::<PathBuf>::new());
    assert_eq!(make_all_parents(Path::new("test")), Vec::<PathBuf>::new());

    assert_eq!(
        make_all_parents(Path::new("test/inner")),
        vec![PathBuf::from("test")]
    );
    assert_eq!(
        make_all_parents(Path::new("test/inner/deep")),
        vec![PathBuf::from("test"), PathBuf::from("test/inner")]
    );
}
