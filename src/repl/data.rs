use super::*;

impl Default for ReplData {
	fn default() -> Self {
		let lib = SourceFile::lib();
		let lib_path = lib.path.clone();
		let mut map = HashMap::new();
		map.insert(lib_path.clone(), lib);

		let mut r = ReplData {
			cmdtree: Builder::new("papyrus")
				.into_commander()
				.expect("empty should pass"),
			file_map: map,
			current_file: lib_path,
			name: "papyrus",
			prompt_colour: Color::Cyan,
			out_colour: Color::BrightGreen,
			compilation_dir: default_compile_dir(),
			linking: LinkingConfiguration::default(),
			redirect_on_execution: true,
		};

		r.with_cmdtree_builder(Builder::new("papyrus"))
			.expect("should build fine")
	}
}

impl ReplData {
	pub fn with_compilation_dir<P: AsRef<Path>>(mut self, dir: P) -> io::Result<Self> {
		let dir = dir.as_ref();
		if !dir.exists() {
			fs::create_dir_all(dir)?;
		}
		assert!(dir.is_dir());
		self.compilation_dir = dir.to_path_buf();
		Ok(self)
	}

	/// Uses the given `Builder` as the root of the command tree.
	/// The builder is amended with the `esc` command at the root, an error will be
	/// returned if the command already exists.
	pub fn with_cmdtree_builder(
		mut self,
		builder: Builder<'static, CommandResult>,
	) -> Result<Self, BuildError> {
		let cmdr = builder
			.root()
			.add_action("esc", "Cancels more input", |_| CommandResult::CancelInput)
			.into_commander()?;

		self.cmdtree = cmdr;
		Ok(self)
	}

	/// Specify that the repl will link an external crate reference.
	/// Overwrites previously specified crate name.
	/// Uses `ReplData.compilation_dir` to copy `rlib` file into.
	///
	/// [See documentation.](https://kurtlawrence.github.io/papyrus/repl/linking.html)
	pub fn with_extern_crate(
		&mut self,
		crate_name: &'static str,
		rlib_path: Option<&str>,
	) -> io::Result<()> {
		self.linking = std::mem::replace(&mut self.linking, LinkingConfiguration::default())
			.link_external_crate(&self.compilation_dir, crate_name, rlib_path)?;
		Ok(())
	}

	pub fn linking(&self) -> &LinkingConfiguration {
		&self.linking
	}

	/// Not meant to used by developer. Use the macros instead.
	/// [See _linking_ module](../pfh/linking.html)
	pub fn set_data_type(mut self, data_type: &str) -> Self {
		self.linking = self.linking.with_data(data_type);
		self
	}
}
