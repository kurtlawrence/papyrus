//! Pertains to compiling a working directory into a library, then executing a function in that library.

mod build;
mod construct;
mod execute;

pub use self::build::{compile, unshackle_library_file, CompilationError};
pub use self::construct::build_compile_dir;
pub(crate) use self::execute::exec;

/// The library name to compile as.c
const LIBRARY_NAME: &str = "papyrus_mem_code";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::*;
    use crate::linking::{Extern, LinkingConfiguration};
    use ::kserd::Kserd;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn nodata_build_fmt_compile_eval_test() {
        let compile_dir = "target/testing/nodata_build_fmt_compile_eval_test";
        let files = vec![pass_compile_eval_file()].into_iter().collect();
        let linking_config = LinkingConfiguration::default();

        // build
        build_compile_dir(&compile_dir, &files, &linking_config).unwrap();
        assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
            .unwrap()
            .contains("\nlet out0 = 2+2;"));

        // compile
        let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

        // eval
        let r = exec::<_, _>(path, "_lib_intern_eval", &()).unwrap(); // execute library fn

        assert_eq!(r.0, Kserd::new_num(4));
    }

    #[test]
    fn brw_data_build_fmt_compile_eval_test() {
        let compile_dir = "target/testing/brw_data_build_fmt_compile_eval_test";
        let files = vec![pass_compile_eval_file()].into_iter().collect();
        let mut linking_config = LinkingConfiguration::default();
        linking_config.external_libs.insert(
            Extern::new("test-resources/external_crate/target/debug/libexternal_crate.rlib")
                .unwrap(),
        );

        // build
        build_compile_dir(&compile_dir, &files, &linking_config).unwrap();
        assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
            .unwrap()
            .contains("\nlet out0 = 2+2;"));

        // compile
        let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

        // eval
        let r = exec::<_, _>(path, "_lib_intern_eval", &()).unwrap(); // execute library fn

        assert_eq!(r.0, Kserd::new_num(4));
    }

    #[test]
    fn mut_brw_data_build_fmt_compile_eval_test() {
        let compile_dir = "target/testing/mut_brw_data_build_fmt_compile_eval_test";
        let files = vec![pass_compile_eval_file()].into_iter().collect();
        let mut linking_config = LinkingConfiguration::default();
        linking_config.external_libs.insert(
            Extern::new("test-resources/external_crate/target/debug/libexternal_crate.rlib")
                .unwrap(),
        );

        // build
        build_compile_dir(&compile_dir, &files, &linking_config).unwrap();
        assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
            .unwrap()
            .contains("\nlet out0 = 2+2;"));

        // // fmt
        // assert!(fmt(&compile_dir));
        // assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
        // 	.unwrap()
        // 	.contains("\n    let out0 = 2 + 2;")); // should be tabbed in (once, unless i wrap it more)

        // compile
        let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

        // eval
        let r = exec::<_, _>(path, "_lib_intern_eval", &()).unwrap(); // execute library fn

        assert_eq!(r.0, Kserd::new_num(4));
    }

    #[test]
    fn exec_and_redirect_test() {
        let compile_dir = "target/testing/exec_and_redirect_test";
        let files = vec![pass_compile_eval_file()].into_iter().collect();
        let mut linking_config = LinkingConfiguration::default();
        linking_config.external_libs.insert(
            Extern::new("test-resources/external_crate/target/debug/libexternal_crate.rlib")
                .unwrap(),
        );

        // build
        build_compile_dir(&compile_dir, &files, &linking_config).unwrap();
        assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
            .unwrap()
            .contains("\nlet out0 = 2+2;"));

        // // fmt
        // assert!(fmt(&compile_dir));
        // assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
        // 	.unwrap()
        // 	.contains("\n    let out0 = 2 + 2;")); // should be tabbed in (once, unless i wrap it more)

        // compile
        let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

        // eval
        let r = exec(path, "_lib_intern_eval", &()).unwrap(); // execute library fn

        assert_eq!(r.0, Kserd::new_num(4));
    }

    #[test]
    fn fail_compile_test() {
        let compile_dir = "target/testing/fail_compile";
        let files = vec![fail_compile_file()].into_iter().collect();
        let linking_config = LinkingConfiguration::default();

        // build
        build_compile_dir(&compile_dir, &files, &linking_config).unwrap();
        assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
            .unwrap()
            .contains("\nlet out0 = 2+;"));

        // compile
        let r = compile(&compile_dir, &linking_config, |_| ());
        assert!(r.is_err());
        match r.unwrap_err() {
            CompilationError::CompileError(_) => (),
            _ => panic!("expecting CompileError"),
        }
    }

    // TODO enable when not on nightly
    // Maybe look into why it doesn't work on nightly?
    // #[test]
    // fn fail_eval_test() {
    //     let compile_dir = "target/testing/fail_eval_test";
    //     let files = vec![fail_eval_file()];
    //     let linking_config = LinkingConfiguration::default();

    //     // build
    //     build_compile_dir(&compile_dir, files.iter(), &linking_config).unwrap();
    //     assert!(fs::read_to_string(&format!("{}/src/lib.rs", compile_dir))
    //         .unwrap()
    //         .contains("\nlet out0 = panic!(\"eval panic\");"));

    //     // compile
    //     let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

    //     // eval
    //     let r = exec::<_, _, std::io::Sink>(&path, "_lib_intern_eval", &(), None); // execute library fn
    //     assert!(r.is_err());
    //     assert_eq!(r, Err("a panic occured with evaluation"));
    // }

    fn pass_compile_eval_file() -> (PathBuf, SourceCode) {
        let mut code = SourceCode::new();
        code.stmts.push(StmtGrp(vec![Statement {
            expr: "2+2".to_string(),
            semi: false,
        }]));
        ("lib".into(), code)
    }

    fn fail_compile_file() -> (PathBuf, SourceCode) {
        let mut code = SourceCode::new();
        code.stmts.push(StmtGrp(vec![Statement {
            expr: "2+".to_string(),
            semi: false,
        }]));
        ("lib".into(), code)
    }

    fn fail_eval_file() -> (PathBuf, SourceCode) {
        let mut code = SourceCode::new();
        code.stmts.push(StmtGrp(vec![Statement {
            expr: "panic!(\"eval panic\")".to_string(),
            semi: false,
        }]));
        ("lib".into(), code)
    }

    #[test]
    fn output_externally_linked_type_as_kserd() {
        let compile_dir = "target/testing/output_externally_linked_type_as_kserd";
        let files = vec![{
            let mut code = SourceCode::new();
            code.crates
                .push(CrateType::parse_str("extern crate rand;").unwrap());
            code.stmts.push(StmtGrp(vec![Statement {
                expr: "rand::random::<u8>()".into(),
                semi: false,
            }]));
            code.stmts.push(StmtGrp(vec![Statement {
                expr: "2+2".into(),
                semi: false,
            }]));
            ("lib".into(), code)
        }]
        .into_iter()
        .collect();
        let mut linking_config = LinkingConfiguration::default();
        linking_config.external_libs.insert(
            Extern::new("test-resources/external_kserd/target/debug/libexternal_kserd.rlib")
                .unwrap(),
        );
        linking_config
            .persistent_module_code
            .push_str("use external_kserd::{kserd, rand};");

        // build
        build_compile_dir(&compile_dir, &files, &linking_config).unwrap();
        let filestr = fs::read_to_string(&format!("{}/src/lib.rs", compile_dir)).unwrap();
        assert!(filestr.contains("\nlet out0 = rand::random::<u8>();"));
        assert!(filestr.contains("\nlet out1 = 2+2;"));

        // compile
        let path = compile(&compile_dir, &linking_config, |_| ()).unwrap();

        // eval
        let r = exec::<_, _>(path, "_lib_intern_eval", &()).unwrap(); // execute library fn

        assert_eq!(r.0, Kserd::new_num(4));
    }
}
