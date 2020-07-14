use super::*;
use crate::code::{
    parse_crates_in_file, validate_static_file_path, AddingStaticFileError, ModsMap, SourceCode,
};

impl<Data> Default for ReplData<Data> {
    fn default() -> Self {
        let lib_path = PathBuf::from("lib");
        let mut map = ModsMap::new();
        map.insert(lib_path.clone(), SourceCode::default());

        let mut r = ReplData {
            cmdtree: Builder::new("papyrus")
                .into_commander()
                .expect("empty should pass"),
            mods_map: map,
            current_mod: lib_path,
            prompt_colour: Color::Cyan,
            out_colour: Color::BrightGreen,
            compilation_dir: default_compile_dir(),
            linking: LinkingConfiguration::default(),
            editing: None,
            editing_src: None,
            static_files: StaticFiles::new(),
            loadedlibs: VecDeque::new(),
            loaded_libs_size_limit: 0,
        };

        r.with_cmdtree_builder(Builder::new("papyrus"))
            .expect("should build fine");

        r
    }
}

impl<Data> ReplData<Data> {
    /// Set the compilation directory. The default is set to `$HOME/.papyrus`.
    pub fn with_compilation_dir<P: AsRef<Path>>(&mut self, dir: P) -> io::Result<&mut Self> {
        let dir = dir.as_ref();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        assert!(dir.is_dir());
        self.compilation_dir = dir.to_path_buf();
        Ok(self)
    }

    /// Link an external library.
    ///
    /// This is primarily used for linking the calling library, and there
    /// is a function on `Extern` to work this path out. It is better to
    /// use `crates.io` than linking libraries, but this method allows for
    /// linking libraries not on `crates.io`.
    ///
    /// [See _linking_ module](../pfh/linking.html)
    pub fn with_external_lib(&mut self, lib: linking::Extern) -> &mut Self {
        self.linking.external_libs.insert(lib);
        self
    }

    /// The current mod that is being repl'd on.
    pub fn current_mod(&self) -> &Path {
        self.current_mod.as_path()
    }

    /// The current source code, this is short hand for
    /// `self.mods_map().get(self.current_mod()).unwrap()`.
    pub fn current_src(&self) -> &SourceCode {
        self.mods_map
            .get(self.current_mod())
            .expect("thin shouldn't fail, always should exist.")
    }

    /// The current file map, mappings of modules to source code.
    pub fn mods_map(&self) -> &ModsMap {
        &self.mods_map
    }

    /// The current linking configuration.
    /// Not mutable as it could lead to undefined behaviour if changed.
    pub fn linking(&self) -> &LinkingConfiguration {
        &self.linking
    }

    /// A mutable reference to the persistent module code.
    ///
    /// This code gets written to each module and can be used to create generic imports. It is also
    /// specifically used to solve _dependency duplication_ if an external library is being linked.
    /// Dependency duplication is discussed in the [_linking_ module](crate::linking).
    pub fn persistent_module_code(&mut self) -> &mut String {
        &mut self.linking.persistent_module_code
    }

    /// The current static files.
    pub fn static_files(&self) -> &StaticFiles {
        &self.static_files
    }

    /// Add a static file.
    ///
    /// The code will be written to disk. The path must be valid, and as they are used for module
    /// paths, must be valid identifiers. See [`StaticFile`](crate::code::StaticFile).
    pub fn add_static_file(
        &mut self,
        path: PathBuf,
        code: &str,
    ) -> Result<bool, AddingStaticFileError> {
        validate_static_file_path(&path).map_err(AddingStaticFileError::InvalidPath)?;

        let hash: [u8; 32] = blake3::hash(code.as_bytes()).into();

        let change = {
            self.static_files
                .get(path.as_path())
                .map(|sf| sf.codehash.as_ref() != &hash)
                .unwrap_or(true)
        };

        if change {
            // parse for crates
            let (code, crates) = parse_crates_in_file(code);
            // write remaining code to disk
            let file_name = self.static_file_name(&path);
            let parent = file_name.parent().expect("should exist");
            fs::create_dir_all(parent).map_err(AddingStaticFileError::Io)?;
            fs::write(file_name, code).map_err(AddingStaticFileError::Io)?;
            // add/overwrite in set
            self.static_files.insert(StaticFile {
                path,
                codehash: Box::new(hash),
                crates,
            });
        }

        Ok(change)
    }

    /// Remove a static file.
    ///
    /// Returns true if the path existed. Any io errors are swallowed.
    pub fn remove_static_file<P: AsRef<Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        let removed = self.static_files.remove(path);
        if removed {
            fs::remove_file(self.static_file_name(path)).ok(); // swallow error
        }
        removed
    }

    fn static_file_name(&self, path: &Path) -> PathBuf {
        self.compilation_dir.join("src").join(path)
    }

    /// Clears the cached loaded libraries.
    ///
    /// This can be used to clear resources. Loaded libraries are stored up to the
    /// [`loaded_libs_size_limit`] but can be cleared earlier if need be.
    ///
    /// [`loaded_libs_size_limit`]: ReplData
    pub fn clear_loaded_libs(&mut self) {
        self.loadedlibs.clear()
    }

    /// Not meant to used by developer. Use the macros instead.
    /// [See _linking_ module](../pfh/linking.html)
    ///
    /// # Safety
    /// Incorrect matching of type will cause undefined behaviour when the REPL evaluates. It will
    /// most likely segfault. Use is not recommended, rather there are macros that correctly map
    /// the type across which are intended for use.
    pub unsafe fn set_data_type(mut self, data_type: &str) -> Self {
        self.linking = self.linking.with_data(data_type);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_files_test() {
        let mut data: ReplData<()> = ReplData::default();
        data.with_compilation_dir("./target/static-files-test")
            .unwrap();
        let r = data
            .add_static_file("name.rs".into(), "let a = 1;")
            .unwrap();
        assert_eq!(r, true);
        let r = data
            .add_static_file("name.rs".into(), "let a = 1;")
            .unwrap();
        assert_eq!(r, false); // unchanged
        let r = data
            .add_static_file("name.rs".into(), "let b = 1;")
            .unwrap();
        assert_eq!(r, true); // changed
        let r = data.remove_static_file("name.rs");
        assert_eq!(r, true);
        // can build paths
        data.add_static_file("path/to/something.rs".into(), "")
            .unwrap();
    }
}
