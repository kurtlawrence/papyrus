use super::*;

#[test]
fn load_rs_source() {
    let mut repl = Repl::new();
    for src_file in RS_FILES.iter() {
        let file = format!("test-src/{}", src_file);
        println!("{}", file);
        let res = load_and_parse(&file);
        match res {
            InputResult::Program(input) => {
                let additionals = build_additionals(input, repl.statements.len());
                let src = repl.build_source(additionals);
                let eval = eval(
                    &format!("test/{}", src_file.split(".").nth(0).unwrap()),
                    src,
                    false,
                );
                let b = eval.is_ok();
                if let Err(e) = eval {
                    println!("{}", e);
                }
                assert!(b);
            }
            InputResult::InputError(e) => {
                println!("{}", e);
                panic!("should have parsed as program, got input error")
            }
            InputResult::More => panic!("should have parsed as program, got more"),
            InputResult::Command(_, _) => panic!("should have parsed as program, got command"),
            InputResult::Empty => panic!("should have parsed as program, got empty"),
            InputResult::Eof => panic!("should have parsed as program, got Eof"),
        }
    }
}

#[test]
fn load_rscript_script() {
    let mut repl = Repl::new();
    for src_file in RSCRIPT_FILES.iter() {
        let file = format!("test-src/{}", src_file);
        println!("{}", file);
        let res = load_and_parse(&file);
        match res {
            InputResult::Program(input) => {
                let additionals = build_additionals(input, repl.statements.len());
                let src = repl.build_source(additionals);
                let eval = eval(
                    &format!("test/{}", src_file.split(".").nth(0).unwrap()),
                    src,
                    false,
                );
                let b = eval.is_ok();
                if let Err(e) = eval {
                    println!("{}", e);
                }
                assert!(b);
            }
            InputResult::InputError(e) => {
                println!("{}", e);
                panic!("should have parsed as program, got input error")
            }
            InputResult::More => panic!("should have parsed as program, got more"),
            InputResult::Command(_, _) => panic!("should have parsed as program, got command"),
            InputResult::Empty => panic!("should have parsed as program, got empty"),
            InputResult::Eof => panic!("should have parsed as program, got Eof"),
        }
    }
}
