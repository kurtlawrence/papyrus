//! Linking an external crate and sharing data.
//!
//! When running a repl you might want to link an external crate. The specific use case is a developer wants to link the crate they are working on into the repl for the user to be able to use. A developer might also want to make data available to the repl. Papyrus has this functionality but makes some assumptions that the developer will need to be aware of, detailed below. When linking is desired, there are two main aspects to consider, the crate name to link and the data transferrence scheme.
//!
//! ## Data Transfer
//! ---
//!
//! A repl instance should always be created by invoking the macro `repl!()` or `repl_with_term!()`. These macros accept a type ascription (such as `u32`, `String`, `MyStruct`, etc) which defines the generic data constraint of the repl. When an evaluation call is made, a mutable reference of the same type will be required to be passed through. Papyrus uses this data to pass it (across an ffi boundary) for the repl to access.
//!
//! ## Crate Linking
//! ---
//!
//! `ReplData` can linking an external crate at compile time, which is useful if a user wants to pass through data of their own type (`my-crate::MyStruct`). It is best to look at the functions on [`ReplData`](../ReplData.html) for configuring linking.
//!
//! ## Example of Crate Linking
//! ---
//!
//! Let's work on a crate called `some-lib`.
//!
//! ### File Setup
//!
//! ***main.rs***:
//!
//! ```rust, no_run
//! #[macro_use]
//! extern crate papyrus;
//!
//! use papyrus::prelude::*;
//!
//! fn main() {
//!   let mut repl = repl!();
//!   repl.data = repl
//!     .data
//!     .with_extern_crate("some_lib", None)
//!     .expect("failed linking crate");
//!
//!   repl.run(&mut ());
//! }
//! ```
//!
//! ***lib.rs***:
//!
//! ```rust, no_run
//! pub struct MyStruct {
//!   a: i32,
//!   b: i32,
//! }
//!
//! impl MyStruct {
//!   pub fn new(a: i32, b: i32) -> Self {
//!     MyStruct { a, b }
//!   }
//!
//!   pub fn add_contents(&self) -> i32 {
//!     self.a + self.b
//!   }
//! }
//! ```
//!
//! ***Cargo.toml***:
//!
//! ```toml
//! [package]
//! name = "some-lib"
//!
//! ...
//!
//! [lib]
//! name = "some_lib"
//! crate-type = ["rlib", "staticlib"]
//! path = "src/lib.rs" # you may need path to the library
//!
//! [dependencies]
//! papyrus = "*"
//! ...
//! ```
//!
//! Notice that you will have to specify the library with a certain `crate-type`. Papyrus links using an `rlib` file, but I have shown that you can also build multiple library files. If you build this project you should find a `libsome_lib.rlib` sitting in your build directory. Papyrus uses this to link when compiling.
//!
//! ### Repl
//!
//! Run this project (`cargo run`). It should spool up fine and prompt you with `papyrus=>`. Now you can try to use the linked crate.
//!
//! ```sh
//! papyrus=> some_lib::MyStruct::new(20, 30).add_contents()
//! papyrus [out0]: 50
//! ```
//!
//! ## What's going on
//! ---
//!
//! - Papyrus takes the crate name you specify and will add this as `extern crate CRATE_NAME;` to the source file.
//! - When setting the external crate name, the `rlib` library is found and copied into the compilation directory.
//!   - Papyrus uses `std::env::current_exe()` to find the executing folder, and searches for the `rlib` file in that folder (`libCRATE_NAME.rlib`)
//!   - Specify the path to the `rlib` library if it is located in a different folder
//! - When compiling the repl code, a rust flag is set, linking the `rlib` such that `extern crate CRATE_NAME;` works.
//!
//! ## Passing `MyStruct` data through
//! ---
//!
//! Keep the example before, but alter the `main.rs` file.
//!
//! ***main.rs***:
//!
//! ```rust, ignore
//! #[macro_use]
//! extern crate papyrus;
//! extern crate some_lib;
//!
//! use some_lib::MyStruct;
//!
//! fn main() {
//!   let mut app_data = MyStruct::new(20, 10);
//!
//!   let mut repl = repl!(some_lib::MyStruct);
//!
//!   repl.data = repl
//!     .data
//!     .with_compilation_dir("test-compilation-area/")
//!     .expect("failed setting compilation dir")
//!     .with_extern_crate("papyrus_extern_test", None)
//!     .expect("failed creating repl data");
//!
//!   repl.run(&mut app_data);
//! }
//! ```
//!
//! Run this project (`cargo run`). It should spool up fine and prompt you with `papyrus=>`. Now you can try to use the linked data. The linked data is in a variable `app_data`, and will always be `app_data: &T`.
//!
//! ```sh
//! papyrus=> app_data.add_contents()
//! papyrus [out0]: 50
//! ```
//!
//! # Notes
//!
//! ## Panics
//!
//! To avoid crashing the application on a panic, `catch_unwind` is employed. This function requires data that crosses the boundary be `UnwindSafe`, making `&` and `&mut` not valid data types. Papyrus uses `AssertUnwindSafe` wrappers to make this work, however it makes `app_data` vunerable to breaking invariant states if a panic is triggered. In practice the repl is designed to be low imapct and such should not have many cases where broken invariants are caused, however there is no guarantee.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{fs, io};

mod macros {
    /// Build a repl instance with the default terminal.
    /// If a type is specfied (ie `repl!(String)`) then the repl will be bounded to use
    /// that data type. Otherwise the default `()` will be used.
    #[macro_export]
    macro_rules! repl {
        // Default Term, with type
        ($type:ty) => {{
            use papyrus;
            let mut r: papyrus::repl::Repl<_, _, $type> = papyrus::repl::Repl::default();
            r.data = unsafe { r.data.set_data_type(&format!("{}", stringify!($type))) };
            r
        }};

        // No data
        () => {{
            use papyrus;
            let r: papyrus::repl::Repl<_, _, ()> = papyrus::repl::Repl::default();
            r
        }};
    }

    /// See `repl!()`.
    #[macro_export]
    macro_rules! repl_with_term {
        // With Term and type
        ($term:expr, $type:ty) => {{
            use papyrus;
            let mut r: papyrus::repl::Repl<_, _, $type> = papyrus::repl::Repl::with_term($term);
            r.data = unsafe { r.data.set_data_type(&format!("{}", stringify!($type))) };
            r
        }};
        // No data with term
        ($term:expr) => {{
            use papyrus;
            let r: papyrus::repl::Repl<_, _, ()> = papyrus::repl::Repl::with_term($term);
            r
        }};
    }
}

/// The external crate and data linking configuration.
pub struct LinkingConfiguration {
    /// Linking data configuration.
    ///
    /// If the user wants to transfer data from the calling application
    /// then it can specify the type of data as a string.
    /// The string must include library and module path, unless accessible
    /// from std library.
    ///
    /// Example: `MyStruct` under the module `some_mod` in crate `some_lib`
    /// - will add `some_lib::some_mod::MyStruct` to the function argument
    /// - function looks like `fn(app_data: &some_lib::some_mod::MyStruct)`
    pub data_type: Option<String>,

    /// Flag whether to prepend `mut` to fn signature (ie `app_data: &mut data_type`).
    /// Indicates a mutable block.
    pub mutable: bool,

    /// Additional external libraries to link.
    ///
    /// These are only precompiled libraries, it is preferable
    /// to link dependencies using `crates.io`.
    ///
    /// The set contains the library names, such as `rand`.
    pub external_libs: HashSet<Extern>,
}

impl Default for LinkingConfiguration {
    fn default() -> Self {
        Self {
            data_type: None,
            mutable: false,
            external_libs: HashSet::new(),
        }
    }
}

impl LinkingConfiguration {
    /// Set the data type. Must be fully qualified from the crate level.
    ///
    /// ## Unsafety
    /// This **must** match the type that is passed through.
    pub unsafe fn with_data(mut self, type_name: &str) -> Self {
        self.data_type = Some(type_name.to_string());
        self
    }

    /// Constructs the function arguments signature.
    pub fn construct_fn_args(&self) -> String {
        self.data_type
            .as_ref()
            .map(|d| {
                if self.mutable {
                    format!("app_data: &mut {}", d)
                } else {
                    format!("app_data: &{}", d)
                }
            }) // matches pfh::compile::execute::DataFunc definition.
            .unwrap_or(String::new())
    }
}

/// Represents an externally linked library.
///
/// The structure holds a path to an `lib*.rlib` library. The path
/// is validated upon construction. To ensure the compilation works,
/// the `deps` folder that is produced on a build must also exist in the
/// same folder as the library.
pub struct Extern {
    /// Path to rlib.
    path: PathBuf,
    alias: Option<&'static str>,
}

impl Extern {
    /// Constructs a new `Extern`al crate linkage.
    ///
    /// Validates the path and dependency folder. The file name must be of
    /// the format `lib*.rlib`, such that `*` is the library name. In the
    /// same folder that the library exists, there _must_ be a `deps` folder,
    /// even if there is no dependencies. This gets validated as well. The
    /// file must exist on disk.
    pub fn new<P: AsRef<Path>>(rlib_path: P) -> io::Result<Self> {
        Self::ctor(rlib_path, None)
    }

    /// Constructs a new `Extern`al crate linkage, with an alias for the lib name;
    ///
    /// Validates the path and dependency folder. The file name must be of
    /// the format `lib*.rlib`, such that `*` is the library name. In the
    /// same folder that the library exists, there _must_ be a `deps` folder,
    /// even if there is no dependencies. This gets validated as well. The
    /// file must exist on disk.
    pub fn with_alias<P: AsRef<Path>>(rlib_path: P, alias: &'static str) -> io::Result<Self> {
        Self::ctor(rlib_path, Some(alias))
    }

    /// Uses the executable name to derive the library name, and
    /// returns the external linking using this. _The executable and library
    /// must be in the same folder_.
    ///
    /// This is a conveniance function if the library name is the same
    /// as the executeable.
    pub fn from_current_exe() -> io::Result<Self> {
        let exe = std::env::current_exe()?;

        let name = exe
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| {
                if cfg!(windows) {
                    s.trim_end_matches(".exe")
                } else {
                    s
                }
            })
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                "failed getting executable name",
            ))?;

        let path = get_rlib_path(name)?;

        Self::new(path)
    }

    fn ctor<P: AsRef<Path>>(rlib_path: P, alias: Option<&'static str>) -> io::Result<Self> {
        let path = rlib_path.as_ref();

        let path = path.canonicalize()?;

        if !path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} not a file on disk", path.display()),
            ));
        }

        let lib = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} does not have file name", path.display()),
            ))?;

        if !lib.starts_with("lib") || !lib.ends_with(".rlib") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "library must be in format lib*.rlib",
            ));
        }

        if lib[3..lib.len() - 5].len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "library has empty name",
            ));
        }

        let deps = path.parent().expect("should have parent").join("deps");
        if !deps.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} not a directory on disk", deps.display()),
            ));
        }

        let e = Extern { path, alias };

        Ok(e)
    }

    /// The library name. This is the `*` in `lib*.rlib`.
    pub fn lib_name(&self) -> &str {
        let lib = self.path.file_name().and_then(|s| s.to_str()).unwrap(); // this has been validated

        &lib[3..lib.len() - 5]
    }

    /// The alias, is there is one.
    pub fn alias(&self) -> Option<&'static str> {
        self.alias.clone()
    }

    /// The canoncialized library path (in `lib*.rlib` format).
    pub fn lib_path(&self) -> &Path {
        self.path.as_path()
    }

    /// The canoncialized `deps` folder which lives in same directory as rlib.
    pub fn deps_path(&self) -> PathBuf {
        self.path.parent().unwrap().join("deps") // this has been validated already.
    }
}

impl PartialEq for Extern {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Extern {}

impl std::hash::Hash for Extern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

fn get_rlib_path(crate_name: &str) -> io::Result<PathBuf> {
    let lib_name = format!("lib{}.rlib", crate_name);
    let exe = std::env::current_exe()?;
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

#[test]
fn get_rlib_path_test() {
    use std::error::Error;
    let r = get_rlib_path("some_crate");
    assert!(r.is_err());
    let e = r.unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::NotFound);
    assert_eq!(e.description(), "did not find file: 'libsome_crate.rlib'");
}
