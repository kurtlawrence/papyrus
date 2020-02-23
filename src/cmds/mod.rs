//! Extendable commands for REPL.
//!
//! The REPL makes use of the crate [`cmdtree`](https://crates.io/crates/cmdtree) to handle commands
//! that can provide additional functionality over just a Rust REPL.
//! A command is prefixed by a colon (`:`) and a number of defaults. To see the commands that are
//! included, type `:help`.
//!
//! # Common Commands
//!
//! There are three common commands, `help`, `cancel` or `c`, and `exit`, which can be invoked in any
//! class.
//!
//! | cmd      | action                                          |
//! | -------- | ----------------------------------------------- |
//! | `help`   | displays help information for the current class |
//! | `cancel` | moves the class back to the command root        |
//! | `exit`   | quit the REPL                                   |
//!
//! Other commands are context based off the command tree, they can be invoked with something similar
//! to `a nested command action` syntax.
//!
//! # Extending Commands
//! ## Setup
//!
//! This tutorial works through the example at
//! [`papyrus/examples/custom-cmds.rs`](https://github.com/kurtlawrence/papyrus/blob/master/papyrus/examples/custom-cmds.rs).
//!
//! To begin, start a binary project with the following scaffolding in the main source code. We define
//! a `custom_cmds` function that will be used to build our custom commands. To highlight the
//! versatility of commands, the REPL is configured to have a persistent app data through a `String`.
//! Notice also the method to alter the prompt name through the `Builder::new` method.
//!
//! ```rust,no_run
//! #[macro_use]
//! extern crate papyrus;
//!
//! use papyrus::cmdtree::{Builder, BuilderChain};
//! use papyrus::cmds::CommandResult;
//!
//! # #[cfg(not(feature = "runnable"))]
//! # fn main() {}
//!
//! # #[cfg(feature = "runnable")]
//! fn main() {
//!     // Build a REPL that will use a String as the persistent app_data.
//!     let mut repl = repl!(String);
//!
//!     // Inject our custom commands.
//!     repl.data.with_cmdtree_builder(custom_cmds()).unwrap();
//!
//!     // Create the persistent data.
//!     let mut app_data = String::new();
//!
//!     // Run the REPL and collect all the output.
//!     let output = repl.run(papyrus::run::RunCallbacks::new(&mut app_data)).unwrap();
//!
//!     // Print the output.
//!     println!("{}", output);
//! }
//!
//! // Define our custom commands.
//! // The CommandResult takes the same type as the app_data,
//! // in this instance it is a String. We could define it as
//! // a generic type but then it loses resolution to interact with
//! // the app_data through commands.
//! fn custom_cmds() -> Builder<CommandResult<String>> {
//!     // The string defines the name and the prompt that will be used.
//!     Builder::new("custom-cmds-app")
//! }
//! ```
//!
//! ## Echo
//!
//! Let's begin with a simple echo command. This command takes the data after the command and prints it
//! to screen. All these commands will be additions to the `Builder::new`.
//! Adding the following action with `add_action` method, the arguments are written to the `Write`able
//! `writer`. The REPL provides the writer and so captures the output. `args` is passed through as a
//! slice of string slices, `cmdtree` provides this, and are always split on word boundaries.
//! Finally, `CommandResult::Empty` is returned which `papyrus` further processes. `Empty` won't do
//! anything but the API provides alternatives.
//!
//! ```rust
//! # extern crate papyrus;
//! # use papyrus::cmdtree::BuilderChain;
//! # use papyrus::cmds::CommandResult;
//! # type Builder = papyrus::cmdtree::Builder<CommandResult<String>>;
//! Builder::new("custom-cmds-app")
//!     .add_action("echo", "repeat back input after command", |writer, args| {
//!     writeln!(writer, "{}", args.join(" ")).ok();
//!     CommandResult::Empty
//!     })
//!     .unwrap()
//! # ;
//! ```
//!
//! Now when the binary is run the REPL runs as usual. If `:help` is entered you should see the
//! following output.
//!
//! ```text
//! [lib] custom-cmds-app=> :help
//! help -- prints the help messages
//! cancel | c -- returns to the root class
//! exit -- sends the exit signal to end the interactive loop
//! Classes:
//!     edit -- Edit previous input
//!     mod -- Handle modules
//! Actions:
//!     echo -- repeat back input after command
//!     mut -- Begin a mutable block of code
//! [lib] custom-cmds-app=>
//! ```
//!
//! The `echo` command exists as a root level action, with the help message displayed. Try calling
//! `:echo Hello, world!` and see what it does!
//!
//!
//! ## Alter app data
//!
//! To extend what the commands can do, lets create a command set that can convert the persistent app
//! data case.
//! The actual actions are nested under a 'class' named `case`. This means to invoke the action, one
//! would call it through `:case upper` or `:case lower`.
//!
//! ```rust
//! # extern crate papyrus;
//! # use papyrus::cmdtree::BuilderChain;
//! # use papyrus::cmds::CommandResult;
//! # type Builder = papyrus::cmdtree::Builder<CommandResult<String>>;
//! Builder::new("custom-cmds-app")
//!     .add_action("echo", "repeat back input after command", |writer, args| {
//!     writeln!(writer, "{}", args.join(" ")).ok();
//!     CommandResult::Empty
//!     })
//!     .begin_class("case", "change case of app_data")
//!     .add_action("upper", "make app_data uppercase", |_, _|
//!     CommandResult::<String>::app_data_fn(|app_data, _repldata, _| {
//!         *app_data = app_data.to_uppercase();
//!         String::new()
//!         })
//!     )
//!         .add_action("lower", "make app_data lowercase", |_, _|
//!     CommandResult::<String>::app_data_fn(|app_data, _repldata, _| {
//!         *app_data = app_data.to_lowercase();
//!         String::new()
//!         })
//!     )
//!     .end_class()
//!     .unwrap()
//! # ;
//! ```
//!
//! An example output is below. To inject some data into the persistent app data, a mutable code block
//! must be entered first.
//!
//! ```text
//! [lib] papyrus=> :mut
//! beginning mut block
//! [lib] custom-cmds-app-mut=> app_data.push_str("Hello, world!")
//! finished mutating block: ()
//! [lib] custom-cmds-app=> app_data.as_str()
//! custom-cmds-app [out0]: "Hello, world!"
//! [lib] custom-cmds-app=> :case upper
//! [lib] custom-cmds-app=> app_data.as_str()
//! custom-cmds-app [out1]: "HELLO, WORLD!"
//! [lib] custom-cmds-app=> :case lower
//! [lib] custom-cmds-app=> app_data.as_str()
//! custom-cmds-app [out2]: "hello, world!"
//! ```
use super::*;
use crate::repl::{Editing, EditingIndex, ReplData};
use cmdtree::{BuildError, Builder, BuilderChain, Commander};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub use cmdtree::Builder as CommandBuilder;

/// The action to take. Passes through a mutable reference to the `ReplData`.
///
/// Use [`CommandResult::repl_data_fn`](CommandResult::repl_data_fn) for convenience.
pub type ReplDataAction<D> = Box<dyn Fn(&mut ReplData<D>, &mut dyn Write) -> String>;

/// The action to take. Passes through a mutable reference to the data `D` _and_ the `ReplData<D>`.
///
/// > _Mutably borrows_ `D` such that a lock must be taken. Use only when necessary.
///
/// Use [`CommandResult::app_data_fn`](CommandResult::app_data_fn) for convenience.
pub type AppDataAction<D> = Box<dyn Fn(&mut D, &mut ReplData<D>, &mut dyn Write) -> String>;

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
    /// Take an action on data `D` and/or `ReplData`.
    ActionOnAppData(AppDataAction<D>),
    /// A blank variant with no action.
    Empty,
}

impl<D> CommandResult<D> {
    /// Convenience function boxing an action on app data.
    ///
    /// > _Mutably borrows_ `D` such that a lock must be taken. Use only when necessary.
    pub fn app_data_fn<F>(func: F) -> Self
    where
        F: 'static + Fn(&mut D, &mut ReplData<D>, &mut dyn Write) -> String,
    {
        CommandResult::ActionOnAppData(Box::new(func))
    }

    /// Convenience function boxing an action on repl data.
    pub fn repl_data_fn<F>(func: F) -> Self
    where
        F: 'static + Fn(&mut ReplData<D>, &mut dyn Write) -> String,
    {
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
        .begin_class("static-files", "Handle static files")
        .add_action(
            "add",
            "Import a static file. args: file-path",
            |wtr, args| add_static_file(wtr, args),
        )
        .add_action("rm", "Remove a static file", |wtr, args| {
            rm_static_file(wtr, args)
        })
        .add_action("ls", "List imported static files", |_, _| ls_static_files())
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
            Ok(ei) => CommandResult::EditReplace(ei, args[1..].iter().copied().collect::<String>()),
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
        data.mods_map.entry(x).or_default();
    }

    data.current_mod = path.to_path_buf();

    ""
}

fn add_static_file<D>(wtr: &mut dyn Write, args: &[&str]) -> CommandResult<D> {
    if let Some(&path) = args.get(0) {
        let pathbuf = PathBuf::from(path);
        match fs::read_to_string(path) {
            Ok(s) => CommandResult::repl_data_fn(move |data, _| {
                data.add_static_file(pathbuf.clone(), &s)
                    .map(|_| "imported/overwrote static file".into())
                    .unwrap_or_else(|e| format!("failed to add {}: {}", pathbuf.display(), e))
            }),
            Err(e) => {
                writeln!(wtr, "failed to read {}: {}", path, e).ok();
                CommandResult::Empty
            }
        }
    } else {
        writeln!(wtr, "add expects a file path").ok();
        CommandResult::Empty
    }
}

fn rm_static_file<D>(wtr: &mut dyn Write, args: &[&str]) -> CommandResult<D> {
    if let Some(&path) = args.get(0) {
        let path = PathBuf::from(path);
        CommandResult::repl_data_fn(move |data, _| {
            data.remove_static_file(&path);
            String::from("removed static file")
        })
    } else {
        writeln!(wtr, "rm expects a file path").ok();
        CommandResult::Empty
    }
}

fn ls_static_files<D>() -> CommandResult<D> {
    CommandResult::repl_data_fn(|data, wtr| {
        let sfs = data.static_files();
        if sfs.is_empty() {
            writeln!(wtr, "no static files imported").ok();
        } else {
            for sf in data.static_files() {
                write!(wtr, "{}", sf.path.display()).ok();
                if let Some(name) = crate::code::static_file_mod_name(&sf.path) {
                    write!(wtr, " -> {}", name).ok();
                }
                writeln!(wtr).ok();
            }
        }
        String::new()
    })
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
