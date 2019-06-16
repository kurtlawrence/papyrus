use crate::pfh::*;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Constructs the compile directory.
/// Takes a list of source files and writes the contents to file.
/// Builds `Cargo.toml` using crates found in `SourceFile`.
pub fn build_compile_dir<P: AsRef<Path>>(
    compile_dir: P,
    mods_map: &ModsMap,
    linking_config: &linking::LinkingConfiguration,
) -> io::Result<()> {
    let compile_dir = compile_dir.as_ref();

    let crates = mods_map.iter().flat_map(|kvp| kvp.1.crates.iter());

    // write cargo toml contents
    create_file_and_dir(compile_dir.join("Cargo.toml"))?
        .write_all(cargotoml_contents(LIBRARY_NAME, crates).as_bytes())?;

    let (src_code, _map) = code::construct_source_code(mods_map, linking_config);

    create_file_and_dir(compile_dir.join("src/lib.rs"))?.write_all(src_code.as_bytes())?;

    Ok(())
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

    let p = Path::new("target/testing/foo");
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
