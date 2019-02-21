use std::path::{Path,PathBuf};
use std::{fs, io};

pub struct LinkingConfiguration {
	/// The name of the external crate.
	/// Needs to match what is compiled.
	/// Example: `some_lib`
	/// - will search for `libsome_lib.rlib` in filesystem
	/// - will add `extern crate some_lib;` to source file
	/// - will compile with `--extern some_lib=libsome_lib.rlib` flag
	pub crate_name: &'static str,
}

impl LinkingConfiguration {
	pub fn link_external_crate<P: AsRef<Path>>(compilation_dir: P, crate_name: &'static str,
		rlib_path: Option<&str>) -> io::Result<Self> {
		let rlib_path = match rlib_path {
			Some(p) => PathBuf::from(p),
			None => get_rlib_path(crate_name)?,
		};

		dbg!(&rlib_path);

		fs::copy(
			rlib_path,
			compilation_dir.as_ref()
				.join(&format!("lib{}.rlib", crate_name)),
		)?;

		Ok(LinkingConfiguration {
			crate_name: crate_name,
		})
	}
}



fn get_rlib_path(crate_name: &str) -> io::Result<PathBuf> {
	let lib_name = format!("lib{}.rlib", crate_name);
	let exe = std::env::current_exe()?;
	dbg!(&exe);
	fs::read_dir(exe.parent().expect("files should always have a parent"))?
		.into_iter()
		.filter(|entry| entry.is_ok())
		.map(|entry| entry.expect("filtered some").path())
		.find(|path| path.ends_with(&lib_name))
		.ok_or(io::Error::new(
			io::ErrorKind::NotFound,
			format!("did not find file: '{}'", lib_name),
		))
}
