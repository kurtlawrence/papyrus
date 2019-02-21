mod command;
mod eval;
mod print;
mod read;
mod writer;

use self::command::Commands;
use colored::*;
use input::{InputReader, InputResult};
use linefeed::terminal::Terminal;
use pfh::{
    linking::{ArgumentType, LinkingConfiguration, LinkingDataType},
    CrateType, SourceFile,
};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub use self::command::{CmdArgs, Command};

pub struct ReplData<Term: Terminal> {
    /// The REPL handled commands.
    /// Can be extended.
    /// ```ignore
    /// let mut repl = Repl::new();
    /// repl.commands.push(Command::new("load", CmdArgs::Filename, "load and evaluate file contents as inputs", |args| {
    /// 	args.repl.run_file(args.arg);
    /// }));
    pub commands: Vec<Command<Term>>,
    /// The file map of relative paths.
    pub file_map: HashMap<PathBuf, SourceFile>,
    /// The current editing and executing file.
    pub current_file: PathBuf,
    /// App and prompt text.
    pub name: &'static str,
    /// The colour of the prompt region. ie `papyrus`.
    pub prompt_colour: Color,
    /// The colour of the out component. ie `[out0]`.
    pub out_colour: Color,
    /// The directory for which compilation is done within.
    /// Defaults to `$HOME/.papyrus/`.
    pub compilation_dir: PathBuf,
    /// The external crate linking configuration,
    linking: Option<LinkingConfiguration>,
}

struct ReplTerminal<Term: Terminal> {
    /// The underlying terminal of `input_rdr`, used to directly control terminal
    terminal: Term,
    /// The persistent input reader.
    input_rdr: InputReader<Term>,
}

struct Writer<'a, T: Terminal>(&'a T);

pub struct Read;
pub struct Evaluate {
    result: InputResult,
}
pub struct ManualPrint;
pub struct Print {
    to_print: String,
    /// Specifies whether to print the `[out#]`
    as_out: bool,
}

pub struct Repl<'data, S, Term: Terminal> {
    state: S,
    terminal: ReplTerminal<Term>,
    pub data: &'data mut ReplData<Term>,
}

impl<Term: Terminal> Default for ReplData<Term> {
    fn default() -> Self {
        let lib = SourceFile::lib();
        let lib_path = lib.path.clone();
        let mut map = HashMap::new();
        map.insert(lib_path.clone(), lib);
        let mut r = ReplData {
            commands: Vec::new(),
            file_map: map,
            current_file: lib_path,
            name: "papyrus",
            prompt_colour: Color::Cyan,
            out_colour: Color::BrightGreen,
            compilation_dir: default_compile_dir(),
            linking: None,
        };
        // help
        r.commands.push(Command::new(
            "help",
            CmdArgs::Text,
            "Show help for commands",
            |repl, arg| {
                // colour output
                let output = repl.data.commands.build_help_response(if arg.is_empty() {
                    None
                } else {
                    Some(arg)
                });
                // colour the output here rather than in print section
                let mut wtr = Vec::new();
                output.split("\n").into_iter().for_each(|line| {
                    if !line.is_empty() {
                        if line.starts_with("Available commands") {
                            writeln!(wtr, "{}", line).unwrap();
                        } else {
                            let mut line_split = line.split(" ");
                            writeln!(
                                wtr,
                                "{} {}",
                                line_split
                                    .next()
                                    .expect("expecting multiple elements")
                                    .bright_yellow(),
                                line_split.into_iter().collect::<Vec<_>>().join(" ")
                            )
                            .unwrap();
                        }
                    }
                });

                Ok(repl.print(&String::from_utf8_lossy(&wtr)))
            },
        ));
        // exit
        r.commands.push(Command::new(
            "exit",
            CmdArgs::None,
            "Exit repl",
            |_, _| Err(()), // flag to break
        ));
        // cancel
        r.commands.push(Command::new(
            "cancel",
            CmdArgs::None,
            "Cancels more input",
            |repl, _| Ok(repl.print("cancelled input")),
        ));
        // cancel (with c)
        r.commands.push(Command::new(
            "c",
            CmdArgs::None,
            "Cancels more input",
            |repl, _| Ok(repl.print("cancelled input")),
        ));

        r
    }
}

impl<Term: Terminal> ReplData<Term> {
    pub fn with_compilation_dir<P: AsRef<Path>>(mut self, dir: P) -> io::Result<Self> {
        let dir = dir.as_ref();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        assert!(dir.is_dir());
        self.compilation_dir = dir.to_path_buf();
        Ok(self)
    }

    /// Specify that the repl will link an external crate reference.
    /// Overwrites previously specified crate name.
    /// Uses `ReplData.compilation_dir` to copy `rlib` file into.
    ///
    /// [See documentation](https://kurtlawrence.github.io/papyrus/repl/linking.html)
    pub fn with_external_crate(
        mut self,
        crate_name: &'static str,
        rlib_path: Option<&str>,
    ) -> io::Result<Self> {
        self.linking = Some(LinkingConfiguration::link_external_crate(
            &self.compilation_dir,
            crate_name,
            rlib_path,
        )?);
        Ok(self)
    }

    pub fn with_external_crate_and_data(
        self,
        crate_name: &'static str,
        rlib_path: Option<&str>,
        data_type: &'static str,
        arg_type: ArgumentType,
    ) -> io::Result<Self> {
        let mut r = self.with_external_crate(crate_name, rlib_path)?;
        r.linking.as_mut().unwrap().data_type = Some(LinkingDataType {
            name: data_type,
            arg: arg_type,
        });
        Ok(r)
    }
}

#[derive(Clone)]
struct Additional {
    items: Option<Vec<String>>,
    stmts: Option<AdditionalStatements>,
    crates: Vec<CrateType>,
}

#[derive(Clone)]
struct AdditionalStatements {
    stmts: Vec<String>,
    print_stmt: String,
}

/// `$HOME/.papyrus`
fn default_compile_dir() -> PathBuf {
    dirs::home_dir().unwrap_or(PathBuf::new()).join(".papyrus/")
}

#[test]
fn test_default_compile_dir() {
    let dir = default_compile_dir();
    println!("{}", dir.display());
    assert!(dir.ends_with(".papyrus/"));
    if cfg!(windows) {
        assert!(dir.starts_with("C:\\Users\\"));
    } else {
        assert!(dir.starts_with("/home/"));
    }
}
