use super::*;

#[test]
fn test_unclosed_delimiter() {
	assert_eq!(parse_program("fn foo() {"), InputResult::More);
	assert_eq!(parse_program("("), InputResult::More);
	assert_eq!(parse_program("{"), InputResult::More);
	assert_eq!(parse_program("let a = ("), InputResult::More);
	assert_eq!(parse_program("let a = {"), InputResult::More);
	assert_eq!(parse_program("let a = foo("), InputResult::More);
	assert_eq!(parse_program("let a = \""), InputResult::More);
}

#[test]
fn test_items() {
	assert_eq!(
		parse_program("fn b() {}"),
		InputResult::Program(Input {
			items: vec!["fn b() {}".to_string()],
			stmts: vec![]
		})
	); // Item::Fn
	assert_eq!(
		parse_program("#[derive(Debug)]\nstruct A {\n\tu: u32\n}"),
		InputResult::Program(Input {
			items: vec!["#[derive(Debug)]\nstruct A {\n\tu: u32\n}".to_string()],
			stmts: vec![]
		})
	); // Item::Struct
}

#[test]
fn test_exprs() {
	// Expr::Binary
	assert_eq!(
		parse_program("2+2"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "2+2".to_string(),
				semi: false
			}]
		})
	);
	assert_eq!(
		parse_program("2+2;"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "2+2".to_string(),
				semi: true
			}]
		})
	);
	// Expr::Macro
	assert_eq!(
		parse_program("println!(\"hello\")"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "println!(\"hello\")".to_string(),
				semi: false
			}]
		})
	);
	assert_eq!(
		parse_program("println!(\"hello\");"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "println!(\"hello\")".to_string(),
				semi: true
			}]
		})
	);
	// Expr::Tuple
	assert_eq!(
		parse_program("()"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "()".to_string(),
				semi: false
			}]
		})
	);
	assert_eq!(
		parse_program("();"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "()".to_string(),
				semi: true
			}]
		})
	);
	// Expr::Call
	assert_eq!(
		parse_program("f()"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "f()".to_string(),
				semi: false
			}]
		})
	);
	assert_eq!(
		parse_program("f();"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "f()".to_string(),
				semi: true
			}]
		})
	);
	// LET
	assert_eq!(
		parse_program("let a = 1;"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "let a = 1".to_string(),
				semi: true
			}]
		})
	);
	// Expr::ForLoop
	assert_eq!(
		parse_program("for i in 0..3 {}"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "for i in 0..3 {}".to_string(),
				semi: false
			}]
		})
	);
	// Expr::Path
	assert_eq!(
		parse_program("b"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "b".to_string(),
				semi: false
			}]
		})
	);
	assert_eq!(
		parse_program("b;"),
		InputResult::Program(Input {
			items: vec![],
			stmts: vec![Statement {
				expr: "b".to_string(),
				semi: true
			}]
		})
	);
}
