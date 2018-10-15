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
fn test_exprs() {
	assert_eq!(
		parse_program("2+2"),
		InputResult::Program(Input::Statement("2+2".to_string(), true))
	); // Expr::Binary
	assert_eq!(
		parse_program("let a = 1;"),
		InputResult::Program(Input::Statement("let a = 1;".to_string(), false))
	); // let statement
}
