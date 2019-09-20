Source file and crate contents.

Input is parsed as Rust code using the `syn` crate. `papyrus` does not differentiate the
myriad of classications for the input, rather it categorises them into [`Item`]s, [`Statement`]s,
and [`CrateType`]s.

`papyrus` will parse a string input into a [`Input`], and these aggregate into a [`SourceCode`]
structure, which flattens each input.

# Examples

Building some source code.
```rust
use papyrus::code::*;

let mut src = SourceCode::new();
src.stmts.push(StmtGrp(vec![Statement {
	expr: String::from("let a = 1"),
	semi: true
    },
    Statement {
	expr: String::from("a"),
	semi: false
    }
]));
```

Crates have some more structure around them.
```rust
use papyrus::code::*;

let input = "extern crate a_crate as acrate;";
let cr = CrateType::parse_str(input).unwrap();

assert_eq!(&cr.src_line, input);
assert_eq!(&cr.cargo_name, "a-crate");
```

[`CrateType`]: CrateType
[`Input`]: Input
[`Item`]: Item
[`SourceCode`]: SourceCode
[`Statement`]: Statement
