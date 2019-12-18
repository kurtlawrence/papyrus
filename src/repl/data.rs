use super::*;
use crate::code::{ModsMap, SourceCode};

impl<Data> Default for ReplData<Data> {
    fn default() -> Self {
        let lib_path = PathBuf::from("lib");
        let mut map = ModsMap::new();
        map.insert(lib_path.clone(), SourceCode::new());

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
            redirect_on_execution: true,
            editing: None,
            editing_src: None,
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

    /// Not meant to used by developer. Use the macros instead.
    /// [See _linking_ module](../pfh/linking.html)
    pub unsafe fn set_data_type(mut self, data_type: &str) -> Self {
        self.linking = self.linking.with_data(data_type);
        self
    }
}
