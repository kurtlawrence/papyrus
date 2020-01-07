use super::LIBRARY_NAME;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{error, fmt};

/// Run `rustc` in the given compilation directory.
pub fn compile<P, F>(
    compile_dir: P,
    linking_config: &crate::linking::LinkingConfiguration,
    mut stderr_line_cb: F,
) -> Result<PathBuf, CompilationError>
where
    P: AsRef<Path>,
    F: FnMut(&str),
{
    let compile_dir = compile_dir.as_ref();
    let lib_file = compile_dir.join("target/debug/");
    let lib_file = if cfg!(windows) {
        lib_file.join(format!("{}.dll", LIBRARY_NAME))
    } else if cfg!(target_os = "macos") {
        lib_file.join(format!("lib{}.dylib", LIBRARY_NAME))
    } else {
        lib_file.join(format!("lib{}.so", LIBRARY_NAME))
    };

    let mut args = vec!["rustc".to_owned(), "--".to_owned(), "-Awarnings".to_owned()];

    for external in linking_config.external_libs.iter() {
        args.push("-L".to_owned());
        args.push(format!("dependency={}", external.deps_path().display()));
        args.push("--extern".to_owned());
        args.push(format!(
            "{}={}",
            external.lib_name(),
            external.lib_path().display()
        ));
    }

    let mut child = Command::new("cargo")
        .current_dir(compile_dir)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| CompilationError::NoBuildCommand)?;

    let stderr = {
        let rdr = BufReader::new(child.stderr.as_mut().expect("stderr should be piped"));
        let mut s = String::new();
        for line in rdr.lines() {
            let line = line.unwrap();
            stderr_line_cb(&line);
            s.push_str(&line);
            s.push('\n');
        }
        s
    };

    match child.wait() {
        Ok(ex) => {
            if ex.success() {
                Ok(lib_file)
            } else {
                Err(CompilationError::CompileError(stderr))
            }
        }
        Err(e) => Err(CompilationError::IOError(e)),
    }
}

/// Function to rename the output library file and remove the associated dependency.
///
/// In relation to [#44](https://github.com/kurtlawrence/papyrus/issues/44), loading a library will
/// effectively lock a library file. This is especially pervasive in windows where the `.dll` locks
/// in both the target directory and the inner `deps` folder. The locking makes subsequent
/// compilations fail with io errors.
///
/// To separate the locking from the loading, the compiled library is renamed _then_ loaded. This
/// function renames the specified library with a random UUID. It also deletes the similarly name
/// library in the `deps` folder. _If there is no `deps` folder, or no library inside the folder,
/// the deletion silently fails_. This step is required for Windows but the behaviour is kept
/// standard across platforms.
pub fn unshackle_library_file<P: AsRef<Path>>(libpath: P) -> PathBuf {
    let libpath = libpath.as_ref();
    let lib = libpath.file_name().expect("there should be a file name");
    let depsfile = libpath
        .parent()
        .expect("there will be parent")
        .join("deps")
        .join(lib);
    if depsfile.exists() {
        std::fs::remove_file(depsfile).ok(); // allow deps files removal to fail
    }
    rename_lib_file(libpath).unwrap_or_else(|_| libpath.to_owned())
}

fn rename_lib_file<P: AsRef<Path>>(compiled_lib: P) -> io::Result<PathBuf> {
    let no_parent = PathBuf::new();
    let parent = compiled_lib.as_ref().parent().unwrap_or(&no_parent);
    let name = || format!("papyrus.{}.lib", uuid::Uuid::new_v4().to_hyphenated());
    let mut lib_path = parent.join(&name());
    while lib_path.exists() {
        lib_path = parent.join(&name());
    }
    std::fs::rename(&compiled_lib, &lib_path)?;
    Ok(lib_path)
}

/// Error type for compilation.
#[derive(Debug)]
pub enum CompilationError {
    /// Failed to initialise `cargo build`. Usually because `cargo` is not in your `PATH` or Rust is not installed.
    NoBuildCommand,
    /// A compiling error occured, with the contents of the stderr.
    CompileError(String),
    /// Generic IO errors.
    IOError(io::Error),
}

impl error::Error for CompilationError {}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilationError::NoBuildCommand => {
                write!(f, "cargo build command failed to start, is rust installed?")
            }
            CompilationError::CompileError(e) => write!(f, "{}", e),
            CompilationError::IOError(e) => write!(f, "io error occurred: {}", e),
        }
    }
}

#[test]
fn compilation_error_fmt_test() {
    let e = CompilationError::NoBuildCommand;
    assert_eq!(
        &e.to_string(),
        "cargo build command failed to start, is rust installed?"
    );
    let e = CompilationError::CompileError("compile err".to_string());
    assert_eq!(&e.to_string(), "compile err");
    let ioe = io::Error::new(io::ErrorKind::Other, "test");
    let e = CompilationError::IOError(ioe);
    assert_eq!(&e.to_string(), "io error occurred: test");
}
