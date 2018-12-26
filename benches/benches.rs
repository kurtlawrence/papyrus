#[macro_use]
extern crate criterion;

extern crate papyrus;

use criterion::Criterion;
use papyrus::*;

fn single_evaluations(c: &mut Criterion) {
    // eval_01
    c.bench_function("eval_01", |b| {
        b.iter_with_setup(
            || {
                let mut repl = Repl::new();
                repl.print = false;
                repl.clean();
                repl
            },
            |mut repl| assert_eq!(repl.evaluate("2+2").unwrap(), "4"),
        )
    });
    // eval_10_sing
    c.bench_function("eval_10_sing", |b| {
        b.iter_with_setup(
            || {
                let mut repl = Repl::new();
                repl.print = false;
                repl.clean();
                repl
            },
            |mut repl| assert_eq!(repl.evaluate(STMTS_10).unwrap(), r#""Hello, world!""#),
        )
    });
    // eval_20_sing
    c.bench_function("eval_20_sing", |b| {
        b.iter_with_setup(
            || {
                let mut repl = Repl::new();
                repl.print = false;
                repl.clean();
                repl
            },
            |mut repl| {
                assert_eq!(
                    repl.evaluate(STMTS_20).unwrap(),
                    r#""Hello -1901523676152-76""#
                )
            },
        )
    });
}

fn progressive_evaluation(c: &mut Criterion) {
    // eval_prog
    c.bench_function("eval_prog", |b| {
        let mut repl = Repl::new();
        repl.print = false;
        repl.clean();
        repl.evaluate("2+2").unwrap();
        b.iter(|| assert_eq!(repl.evaluate("2+2").unwrap(), "4"))
    });
}

fn compiling(c: &mut Criterion) {
    // cargo_fs_compile_clean_10
    c.bench_function("cargo_fs_compile_clean_10", |b| {
        let src_file = SourceFile {
            src: format!("{};", STMTS_10),
            file_type: SourceFileType::Rscript,
            file_name: "bench_compile".to_string(),
            crates: Vec::new(),
        };
        b.iter_with_setup(
            || std::fs::remove_dir_all("test/bench-cargo_fs_compile_clean_10").is_ok(),
            |_| {
                Exe::compile(&src_file, "test/bench-cargo_fs_compile_clean_10")
                    .unwrap()
                    .wait()
                    .unwrap()
            },
        )
    });
    // cargo_fs_compile_clean_20
    c.bench_function("cargo_fs_compile_clean_20", |b| {
        let src_file = SourceFile {
            src: format!("{};", STMTS_20),
            file_type: SourceFileType::Rscript,
            file_name: "bench_compile".to_string(),
            crates: Vec::new(),
        };
        b.iter_with_setup(
            || std::fs::remove_dir_all("test/bench-cargo_fs_compile_clean_20").is_ok(),
            |_| {
                Exe::compile(&src_file, "test/bench-cargo_fs_compile_clean_20")
                    .unwrap()
                    .wait()
                    .unwrap()
            },
        )
    });
    // cargo_fs_compile
    c.bench_function("cargo_fs_compile", |b| {
        let src_file = SourceFile {
            src: format!("{};", STMTS_20),
            file_type: SourceFileType::Rscript,
            file_name: "bench_compile".to_string(),
            crates: Vec::new(),
        };
        Exe::compile(&src_file, "test/bench-cargo_fs_compile")
            .unwrap()
            .wait()
            .is_ok();
        b.iter(|| {
            Exe::compile(&src_file, "test/bench-cargo_fs_compile")
                .unwrap()
                .wait()
                .unwrap()
        })
    });
}

fn executing(c: &mut Criterion) {
    // exe_fs_run
    c.bench_function("exe_fs_run", |b| {
        let src_file = SourceFile {
            src: format!("{};", STMTS_20),
            file_type: SourceFileType::Rscript,
            file_name: "bench_compile".to_string(),
            crates: Vec::new(),
        };
        let p = Exe::compile(&src_file, "test/bench-exe_fs_run").unwrap();
        let p = p.wait().unwrap();
        b.iter(|| p.run(&std::env::current_dir().unwrap()))
    });
}

criterion_group! {
name = singles;
config = Criterion::default().sample_size(10);
 targets = single_evaluations
 }

criterion_group! {
name = progressives;
config = Criterion::default().sample_size(15);
 targets = progressive_evaluation, compiling
 }

criterion_group! {
name = defaults;
config = Criterion::default();
 targets = executing
 }

criterion_main!(singles, progressives, defaults);

const STMTS_10: &str = r#"let a = 1;
let b = 2;
let c = a * b;
let c = a * c + 10;
let a = a * b * c;
let mut s = String::from("Hello");
let a = a + b + c;
let c = a - b;
s.push_str(", world!");
s"#;

const STMTS_20: &str = r#"let a = 1;
let b = 2;
let c = a * b;
let c = a * c + 10;
let a = a * b * c;
let mut s = String::from("Hello ");
let a = a + b + c;
let c = a - b;
let d = a + b + c;
let e = a + b + c  + d;
let f = d - e;
let a = a - d - e;
let b = d - f;
s.push_str(&a.to_string());
s.push_str(&b.to_string());
s.push_str(&c.to_string());
s.push_str(&d.to_string());
s.push_str(&e.to_string());
s.push_str(&f.to_string());
s"#;
