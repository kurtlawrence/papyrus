use self::command::Commands;
use super::compile::*;
use super::file::SourceFile;
use super::input::{self, Input, InputReader, InputResult};
use super::*;
use colored::*;
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};
use term_cursor;

mod command;
mod state;

pub use self::command::{CmdArgs, Command};

pub struct ReplData {
    /// The REPL handled commands.
    /// Can be extended.
    /// ```ignore
    /// let mut repl = Repl::new();
    /// repl.commands.push(Command::new("load", CmdArgs::Filename, "load and evaluate file contents as inputs", |args| {
    /// 	args.repl.run_file(args.arg);
    /// }));
    pub commands: Vec<Command>,
    /// Items compiled into every program. These are functions, types, etc.
    pub items: Vec<Vec<String>>,
    /// Blocks of statements applied in order.
    pub statements: Vec<Vec<String>>,
    /// Crates to referenced.
    pub crates: Vec<CrateType>,
    /// Flag whether to print to stdout.
    pub print: bool,
    /// App and prompt text.
    pub name: &'static str,
    /// The colour of the prompt region. ie `papyrus`.
    pub prompt_colour: Color,
    /// The colour of the out component. ie `[out0]`.
    pub out_colour: Color,
}

pub struct Read;
pub struct Evaluate {
    result: InputResult,
}
pub struct ManualPrint;
pub struct Print {
    to_print: String,
    success: bool,
}

pub struct Repl<'data, S> {
    state: S,
    pub data: &'data mut ReplData,
}

impl Default for ReplData {
    fn default() -> Self {
        let mut r = ReplData {
            commands: Vec::new(),
            items: Vec::new(),
            statements: Vec::new(),
            crates: Vec::new(),
            name: "papyrus",
            prompt_colour: Color::Cyan,
            out_colour: Color::BrightGreen,
            print: true,
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
                output.split("\n").into_iter().for_each(|line| {
                    if !line.is_empty() {
                        if line.starts_with("Available commands") {
                            println!("{}", line);
                        } else {
                            let mut line_split = line.split(" ");
                            println!(
                                "{} {}",
                                line_split
                                    .next()
                                    .expect("expecting multiple elements")
                                    .bright_yellow(),
                                line_split.into_iter().collect::<Vec<_>>().join(" ")
                            );
                        }
                    }
                });

                Ok(repl.print("", false))
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
            |repl, _| Ok(repl.print("cancelled input", false)),
        ));
        // cancel (with c)
        r.commands.push(Command::new(
            "c",
            CmdArgs::None,
            "Cancels more input",
            |repl, _| Ok(repl.print("cancelled input", false)),
        ));
        // load
        r.commands.push(Command::new(
            "load",
            CmdArgs::Filename,
            "load *.rs or *.rscript as inputs",
            |repl, arg| {
                let eval = repl.load(arg);
                eval.eval()
            },
        ));

        r
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

fn load_and_parse<P: AsRef<Path>>(file_path: P) -> InputResult {
    match SourceFile::load(file_path) {
        Ok(src) => {
            // add crates back in....
            let src = format!(
                "{}\n{}",
                src.crates.into_iter().fold(String::new(), |mut acc, x| {
                    acc.push_str(&x.src_line);
                    acc.push('\n');
                    acc
                }),
                src.src
            );
            let r = input::parse_program(&src);
            if r == InputResult::More {
                // there is a trailing a semi colon, parse with an empty fn
                debug!("parsing again as there was no returning expression");
                input::parse_program(&format!("{}\n()", src))
            } else {
                r
            }
        }
        Err(e) => InputResult::InputError(e),
    }
}

fn compile_dir() -> PathBuf {
    let dir = dirs::home_dir().unwrap_or(PathBuf::new());
    let dir = PathBuf::from(format!("{}/.papyrus", dir.to_string_lossy()));
    dir
}

fn overwrite_current_console_line(line: &str) {
    if cfg!(test) {
        println!("{}", line);
    } else {
        let (col, row) = term_cursor::get_pos().expect("getting cursor position failed");
        term_cursor::set_pos(0, row).expect("setting cursor position failed");
        for _ in 0..col {
            print!(" ");
        }
        term_cursor::set_pos(0, row).expect("setting cursor position failed");
        print!("{}", line);
        std::io::stdout().flush().expect("flushing stdout failed");
    }
}

fn code(statements: &str, items: &str) -> String {
    format!(
        r#"fn main() {{
    {stmts}
}}

{items}
"#,
        stmts = statements,
        items = items
    )
}
