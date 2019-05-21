use crate::pfh::*;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Constructs the compile directory.
/// Takes a list of source files and writes the contents to file.
/// Builds `Cargo.toml` using crates found in `SourceFile`.
pub fn build_compile_dir<P: AsRef<Path>>(
    compile_dir: P,
    file_map: &HashMap<PathBuf, SourceFile>,
    linking_config: &linking::LinkingConfiguration,
) -> io::Result<()> {
    let compile_dir = compile_dir.as_ref();

    let mut crates = Vec::new();

    // write source files
    for file in file_map.values() {
        // add linked crate if there is one to lib file
        let mut contents = String::new();
        if file.path == Path::new("lib.rs") {
            // add in external crates
            for external in linking_config.external_libs.iter() {
                if let Some(alias) = external.alias() {
                    contents.push_str(&format!(
                        "extern crate {} as {};\n",
                        external.lib_name(),
                        alias
                    ));
                } else {
                    contents.push_str(&format!("extern crate {};\n", external.lib_name()));
                }
            }
        }

        // add in child mods
        for child_mod in find_direct_children(file_map, &file.path) {
            contents.push_str("mod ");
            contents.push_str(child_mod);
            contents.push_str(";\n");
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

fn find_direct_children<'a>(
    file_map: &'a HashMap<PathBuf, SourceFile>,
    path: &Path,
) -> Vec<&'a str> {
    let path = if path == Path::new("lib.rs") {
        Path::new("")
    } else {
        path.parent()
            .expect("there should always be a parent as there is always a /mod.rs")
    };

    file_map
        .iter()
        .filter_map(|kvp| {
            kvp.0.strip_prefix(path).ok().and_then(|p| {
                if p.components().count() == 2 {
                    kvp.1.mod_path.last().map(|x| x.as_str())
                } else {
                    None
                }
            })
        })
        .collect()
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
