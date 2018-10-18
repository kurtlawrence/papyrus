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
		InputResult::Program(Input::Item("fn b() {}".to_string()))
	); // Item::Fn
}

#[test]
fn test_exprs() {
	// Expr::Binary
	assert_eq!(
		parse_program("2+2"),
		InputResult::Program(Input::Statements(vec!["2+2".to_string()], false))
	);
	assert_eq!(parse_program("2+2;"), InputResult::More);
	// Expr::Macro
	assert_eq!(
		parse_program("println!(\"hello\")"),
		InputResult::Program(Input::Statements(
			vec!["println!(\"hello\")".to_string()],
			false
		))
	);
	assert_eq!(parse_program("(println!(\"hello\"));"), InputResult::More);
	// Expr::Tuple
	assert_eq!(
		parse_program("()"),
		InputResult::Program(Input::Statements(vec!["()".to_string()], false))
	);
	assert_eq!(parse_program("();"), InputResult::More);
	// Expr::Call
	assert_eq!(
		parse_program("f()"),
		InputResult::Program(Input::Statements(vec!["f()".to_string()], false))
	);
	assert_eq!(parse_program("f();"), InputResult::More);
	// Expr::Let
	assert_eq!(
		parse_program("let a = 1"),
		InputResult::Program(Input::Statements(vec!["let a = 1".to_string()], false))
	);
	assert_eq!(parse_program("let a = 1;"), InputResult::More);
}
