The repl takes the commands given and evaluates them, setting a local variable such that the data can be continually referenced. To construct a repl instance, use the macros [`repl!`].

Repls are state machines, consisting of a read, evaluate, and print states. Each state will lead directly to the next with relevant methods. Generally a user will only use the `.read` and `.eval` methods. Calling `.run` will consume the repl and block the thread until it exits.

You can replicate the `run` behaviour with a basic implementation:

```rust, ignore
#[macro_use]
extern crate papyrus;

use papyrus::prelude::*;

let mut repl = repl!();

loop {
  let result = repl.read().eval(&mut ());
  match result.signal {
    Signal::None => (),
    Signal::Exit => break,
  }
  repl = result.repl.print();
}
```

There is also the ability to pass data around [(see _linking_)](../linking.html) and run things asynchronously. Take a look at the [github examples](https://github.com/kurtlawrence/papyrus/tree/master/examples) for more implementations and uses of the repl.

## Commands
---

The repl can also pass commands (which are stored in a [cmdtree](https://github.com/kurtlawrence/cmdtree)). Commands are always prefixed by a `.`. Type `.help` for information on commands.

### `.mut` command

A noteworthy command is the `.mut`, which will set the next block able to mutate `app_data`. Mutating is single use, after evaluation is complete, it will not be run again. Any local variables assigned will also not be available. There are examples on github for mutating.

## REPL process
---

Example interaction:

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