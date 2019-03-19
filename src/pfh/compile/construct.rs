use crate::pfh::*;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::{error, fmt, fs};

/// Constructs the compile directory.
/// Takes a list of source files and writes the contents to file.
/// Builds `Cargo.toml` using crates found in `SourceFile`.
pub fn build_compile_dir<'a, P, I>(
    compile_dir: P,
    files: I,
    linking_config: &linking::LinkingConfiguration,
) -> io::Result<()>
where
    P: AsRef<Path>,
    I: Iterator<Item = &'a SourceFile>,
{
    let compile_dir = compile_dir.as_ref();

    let mut crates = Vec::new();

    // write source files
    for file in files {
        // add linked crate if there is one to lib file
        let mut contents = String::new();
        if let Some(cname) = linking_config.crate_name {
            if file.path == Path::new("lib.rs") {
                contents.push_str(&format!("extern crate {};\n", cname));
            }
        }
        contents.push_str(&file::code::construct(
            &file.contents,
            &file.mod_path,
            linking_config,
        ));

        create_file_and_dir(compile_dir.join("src/").join(&file.path))?
            .write_all(contents.as_bytes())?;
        for c in file.contents.iter().flat_map(|x| &x.crates) {
            crates.push(c);
        }
    }

    // write cargo toml contents
    create_file_and_dir(compile_dir.join("Cargo.toml"))?
        .write_all(cargotoml_contents(LIBRARY_NAME, crates.into_iter()).as_bytes())?;

    Ok(())
}

/// Run `cargo fmt` in the given directory.
pub fn fmt<P: AsRef<Path>>(compile_dir: P) -> bool {
    match Command::new("cargo")
        .current_dir(compile_dir)
        .args(&["fmt"])
        .output()
    {
        Ok(output) => output.status.success(),
        Err(e) => {
            debug!("{}", e);
            false
        }
    }
}

/// Creates the specified file along with the directory to it if it doesn't exist.
fn create_file_and_dir<P: AsRef<Path>>(file: P) -> io::Result<fs::File> {
    let file = file.as_ref();
    debug!("trying to create file: {}", file.display());
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::File::create(file)
}

#[test]
fn create_file_and_dir_test() {
    use std::path::Path;

    let p = Path::new("foo.txt");
    assert!(!p.exists());
    create_file_and_dir(&"foo.txt").unwrap();
    assert!(p.exists());
    fs::remove_file(p).unwrap();
    assert!(!p.exists());

    let p = Path::new("test/foo");
    assert!(!p.exists());
    create_file_and_dir(&p).unwrap();
    assert!(p.exists());
    fs::remove_file(p).unwrap();
    assert!(!p.exists());
}

fn cargotoml_contents<'a, I: Iterator<Item = &'a CrateType>>(lib_name: &str, crates: I) -> String {
    format!(
        r#"[package]
name = "{lib_name}"
version = "0.1.0"

[lib]
name = "{lib_name}"
crate-type = [ "cdylib" ]
path = "src/lib.rs"

[dependencies]
{crates}
"#,
        lib_name = lib_name,
        crates = crates
            .map(|c| format!(r#"{} = "*""#, c.cargo_name))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
