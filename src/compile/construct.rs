use super::LIBRARY_NAME;
use crate::{
    code::{self, CrateType, ModsMap, StaticFiles},
    linking,
};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

/// Constructs the compile directory.
/// Takes a list of source files and writes the contents to file.
/// Builds `Cargo.toml` using crates found in `SourceFile`.
pub fn build_compile_dir<P>(
    compile_dir: P,
    mods_map: &ModsMap,
    linking_config: &linking::LinkingConfiguration,
    static_files: &StaticFiles,
) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let compile_dir = compile_dir.as_ref();

    let crates = mods_map
        .iter()
        .flat_map(|kvp| kvp.1.crates.iter())
        .chain(static_files.iter().flat_map(|x| x.crates.iter()));
    let crates = dedup_crates(crates);

    // write cargo toml contents
    create_file_and_dir(compile_dir.join("Cargo.toml"))?
        .write_all(cargotoml_contents(LIBRARY_NAME, crates.into_iter()).as_bytes())?;

    let (src_code, _map) = code::construct_source_code(mods_map, linking_config, static_files);

    create_file_and_dir(compile_dir.join("src/lib.rs"))?.write_all(src_code.as_bytes())?;

    Ok(())
}

fn dedup_crates<'a>(crates: impl Iterator<Item = &'a CrateType>) -> Vec<&'a CrateType> {
    let mut crates: Vec<&CrateType> = crates.collect();
    crates.sort_by_key(|x| &x.cargo_name);
    crates.dedup_by_key(|x| &x.cargo_name);
    crates
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

fn cargotoml_contents<'a, I: Iterator<Item = &'a CrateType>>(lib_name: &str, crates: I) -> String {
    format!(
        r#"[package]
name = "{lib_name}"
version = "0.1.0"
edition = "2018"

[lib]
name = "{lib_name}"
crate-type = [ "cdylib" ]
path = "src/lib.rs"

[dependencies]
kserd = {{ version = "0.3", default-features = false }}
{crates}
"#,
        lib_name = lib_name,
        crates = crates
            .map(|c| format!(r#"{} = "*""#, c.cargo_name))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_dedup_crates() {
        let crates = vec![
            CrateType::parse_str("extern crate rand;").unwrap(),
            CrateType::parse_str("extern crate rand as rnd;").unwrap(),
            CrateType::parse_str("extern crate third;").unwrap(),
        ];
        let crates = dedup_crates(crates.iter());

        let v: Vec<_> = crates.iter().map(|x| &x.cargo_name).collect();
        assert_eq!(&v, &["rand", "third"]);
    }
}
