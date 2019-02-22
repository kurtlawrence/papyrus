use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct LinkingConfiguration<A> {
	/// The name of the external crate.
	/// Needs to match what is compiled.
	/// Example: `some_lib`
	/// - will search for `libsome_lib.rlib` in filesystem
	/// - will add `extern crate some_lib;` to source file
	/// - will compile with `--extern some_lib=libsome_lib.rlib` flag
	pub crate_name: &'static str,
	/// An option to specify if data is to be transferred to the calling function.
	data_type: Option<LinkingDataType<A>>,
}

impl LinkingConfiguration<NoData> {
	pub fn link_external_crate<P: AsRef<Path>>(
		compilation_dir: P,
		crate_name: &'static str,
		rlib_path: Option<&str>,
	) -> io::Result<Self> {
		let rlib_path = match rlib_path {
			Some(p) => PathBuf::from(p),
			None => get_rlib_path(crate_name)?,
		};

		dbg!(&rlib_path);

		fs::copy(
			rlib_path,
			compilation_dir
				.as_ref()
				.join(&format!("lib{}.rlib", crate_name)),
		)?;

		Ok(LinkingConfiguration {
			crate_name: crate_name,
			data_type: None,
		})
	}

	pub fn with_borrowed_data(self, type_name: &str) -> LinkingConfiguration<BorrowData> {
		let r = LinkingConfiguration {
			crate_name: self.crate_name,
			data_type: Some(LinkingDataType {
				name: type_name.to_string(),
				arg: ArgumentType::BorrowData(BorrowData),
			}),
		};

		r
	}

	pub fn with_mut_borrowed_data(self, type_name: &str) -> LinkingConfiguration<BorrowMutData> {
		let r = LinkingConfiguration {
			crate_name: self.crate_name,
			data_type: Some(LinkingDataType {
				name: type_name.to_string(),
				arg: ArgumentType::BorrowMutData(BorrowMutData),
			}),
		};

		r
	}
}

impl<A> LinkingConfiguration<A> {
	pub fn construct_fn_args(&self) -> String {
		match self.data_type {
			Some(ref d) => match d.arg {
				ArgumentType::BorrowData(_) => {
					format!("app_data: &{}::{}", self.crate_name, d.name)
				}
				ArgumentType::BorrowMutData(_) => {
					format!("app_data: &mut {}::{}", self.crate_name, d.name)
				}
			},
			None => "".to_string(),
		}
	}
}

pub struct NoData;
pub struct BorrowData;
pub struct BorrowMutData;

/// Linking data configuration.
/// If the user wants to transfer data from the calling application then it can specify the type of data as a string.
/// The string must include module path if not accesible from the root of the external crate.
/// The `ArgumentType` parameter flags how to pass the data to the function.
///
/// Example: `MyStruct` under the module `some_mod` in crate `some_lib` with `ArgumentType::Borrow`
/// - will add `some_lib::some_mod::MyStruct` to the function argument
/// - function looks like `fn(app_data: &some_lib::some_mode::MyStruct)`
struct LinkingDataType<A> {
	/// The name of the data type.
	/// Needs to be path qualified.
	pub name: String,
	/// The argument type to modify the function structure.
	arg: ArgumentType<A>,
}

enum ArgumentType<A> {
	BorrowData(A),
	BorrowMutData(A),
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
