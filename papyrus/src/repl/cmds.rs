use super::*;
use std::io::Write;

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
                all.push(path.to_path_buf());

                for x in all {
                    if !repl_data.file_map.contains_key(&x) {
                        repl_data.file_map.insert(x, pfh::SourceCode::new());
                    }
                }

                repl_data.current_file = path.to_path_buf();

                String::new()
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

    (1..components.len())
        .into_iter()
        .map(|idx| components[0..idx].iter().collect::<PathBuf>())
        .collect()
}

fn make_path(path: &str) -> Option<PathBuf> {
    let path = path.trim();

    let path = path.replace(".rs", "").replace("mod", "").replace("-", "_");

    if path == "lib" {
        return Some(PathBuf::from("lib"));
    }

    let x: &[_] = &['/', '\\'];
    let path = path.trim_matches(x); // remove starting or trailing slashes

    if path.is_empty() {
        return None;
    }

    Some(PathBuf::from(path))
}

#[test]
fn make_path_test() {
    assert_eq!(make_path("   "), None);

    assert_eq!(make_path("lib"), Some(PathBuf::from("lib")));
    assert_eq!(make_path("lib.rs"), Some(PathBuf::from("lib")));

    assert_eq!(make_path("test"), Some(PathBuf::from("test")));
    assert_eq!(make_path("test/inner"), Some(PathBuf::from("test/inner")));
    assert_eq!(make_path("inner/test"), Some(PathBuf::from("inner/test")));

    assert_eq!(make_path("//"), None);

    assert_eq!(make_path("\\hello\\"), Some(PathBuf::from("hello")));
}

#[test]
fn make_all_parents_test() {
    // only handle parents
    assert_eq!(make_all_parents(Path::new("")), Vec::<PathBuf>::new());
    assert_eq!(make_all_parents(Path::new("test")), Vec::<PathBuf>::new());

    assert_eq!(
        make_all_parents(Path::new("test/inner")),
        vec![PathBuf::from("test")]
    );
    assert_eq!(
        make_all_parents(Path::new("test/inner/deep")),
        vec![PathBuf::from("test"), PathBuf::from("test/inner")]
    );
}
