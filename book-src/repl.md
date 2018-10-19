# REPL

>Back to [SUMMARY](./SUMMARY.md)

The repl takes the commands given and evaluates them, setting a local variable such that the data can be continually referenced.

```sh
papyrus=> let a = 1;
papyrus.> a
papyrus [out0]: 1
papyrus=>
```

Here we define a variable `let a = 1;`. Papyrus knows that the end result is not an expression (given the trailing semi colon) so waits for more input (`.>`). We then give it `a` which is an expression and gets evaluated. If compilation is successful the expression is set to the variable `out0` (where the number will increment with expressions) and then be printed with the `Debug` trait. If an expression evaluates to something that is not `Debug` then you will receive a compilation error. Finally the repl awaits more input `=>`.

> The expression is using `let out# = <expr>;` behind the scenes.

You can also define structures and functions.

```sh
papyrus=> fn a(i: u32) -> u32 {
papyrus.> i + 1
papyrus.> }
papyrus=> a(1)
papyrus [out0]: 2
papyrus=>
```

```txt
papyrus=> #[derive(Debug)] struct A {
papyrus.> a: u32,
papyrus.> b: u32
papyrus.> }
papyrus=> let a = A {a: 1, b: 2};
papyrus.> a
papyrus [out0]: A { a: 1, b: 2 }
papyrus=>
```

Please help if the Repl cannot parse your statements, or help with documentation! [https://github.com/kurtlawrence/papyrus](https://github.com/kurtlawrence/papyrus).