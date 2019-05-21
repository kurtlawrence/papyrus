use super::*;

impl<Data> ReplData<Data> {
    /// Uses the given `Builder` as the root of the command tree.
    ///
    /// An error will be returned if any command already exists.
    pub fn with_cmdtree_builder(
        &mut self,
        builder: Builder<'static, CommandResult<Data>>,
    ) -> Result<&mut Self, BuildError> {
        let cmdr = builder
            .root()
            .add_action("mut", "Begin a mutable block of code", |_, _| {
                CommandResult::BeginMutBlock
            })
            .begin_class("mod", "Handle modules")
            .add_action(
                "switch",
                "Switch to a module, creating one if necessary. switch path/to/module",
                |wtr, args| switch_module(args, wtr),
            )
            .end_class()
            .into_commander()?;

        self.cmdtree = cmdr;
        Ok(self)
    }
}

fn switch_module<D, W: Write>(args: &[&str], mut wtr: W) -> CommandResult<D> {
    if let Some(path) = args.get(0) {
        if let Some(path) = make_path(path) {
            CommandResult::repl_data_fn(move |repl_data, _wtr| {
                let path = &path;

                let mut all = make_all_parents(path);
                all.push(path.clone());

                for x in all {
                    if !repl_data.file_map.contains_key(&x) {
                        repl_data.file_map.insert(x, pfh::SourceCode::new());
                    }
                }

                repl_data.current_file = path.clone();
            })
        } else {
            writeln!(wtr, "failed to parse {} into a valid module path", path).unwrap();
            CommandResult::Empty
        }
    } else {
        writeln!(wtr, "switch expects a path to module argument").unwrap();
        CommandResult::Empty
    }
}

fn make_all_parents(path: &Path) -> Vec<PathBuf> {
    let components: Vec<_> = path.components().collect();

    (1..components.len().saturating_sub(1))
        .into_iter()
        .map(|idx| {
            components[0..idx]
                .iter()
                .collect::<PathBuf>()
                .join("mod.rs")
        })
        .collect()
}

fn make_path(path: &str) -> Option<PathBuf> {
    let path = path.trim();

    if path == "lib" || path == "lib.rs" {
        return Some(PathBuf::from("lib.rs"));
    }

    let path = path.replace(".rs", "").replace("mod", "").replace("-", "_");

    let x: &[_] = &['/', '\\'];
    let path = path.trim_matches(x); // remove starting or trailing slashes

    if path.is_empty() {
        return None;
    }

    Some(Path::new(&path).join("mod.rs"))
}

#[test]
fn make_path_test() {
    assert_eq!(make_path("   "), None);

    assert_eq!(make_path("lib"), Some(PathBuf::from("lib.rs")));
    assert_eq!(make_path("lib.rs"), Some(PathBuf::from("lib.rs")));

    assert_eq!(make_path("test"), Some(PathBuf::from("test/mod.rs")));
    assert_eq!(
        make_path("test/inner"),
        Some(PathBuf::from("test/inner/mod.rs"))
    );
    assert_eq!(
        make_path("inner/test"),
        Some(PathBuf::from("inner/test/mod.rs"))
    );

    assert_eq!(make_path("//"), None);

    assert_eq!(make_path("\\hello\\"), Some(PathBuf::from("hello/mod.rs")));
}

#[test]
fn make_all_parents_test() {
    // only handle parents
    assert_eq!(make_all_parents(Path::new("")), Vec::<PathBuf>::new());
    assert_eq!(make_all_parents(Path::new("test")), Vec::<PathBuf>::new());
    assert_eq!(
        make_all_parents(Path::new("test/mod.rs")),
        Vec::<PathBuf>::new()
    );

    assert_eq!(
        make_all_parents(Path::new("test/inner/mod.rs")),
        vec![PathBuf::from("test/mod.rs")]
    );
    assert_eq!(
        make_all_parents(Path::new("test/inner/deep/mod.rs")),
        vec![
            PathBuf::from("test/mod.rs"),
            PathBuf::from("test/inner/mod.rs")
        ]
    );
}
