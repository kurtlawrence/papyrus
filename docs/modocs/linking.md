Linking an external crate and sharing data.

When running a REPL you might want to link an external crate.
The specific use case is a developer wants to link the crate they are working on
into the REPL for the user to be able to use.
A developer might also want to make data available to the REPL.
Papyrus has this functionality but makes some assumptions that the developer will
need to be aware of, detailed below.

## Worked Example

A REPL instance should always be created by invoking the macro `repl!()`.
In the examples below this will be elided for as the documentation won't compile with the macros.
The macro accepts a type ascription (such as `u32`, `String`, `MyStruct`, etc.) which defines the generic data constraint of the REPL.
When an evaluation call is made, a mutable reference of the same type will be required to be passed through.
Papyrus uses this data to pass it (across an FFI boundary) for the REPL to access.

To show the functionality of linking, let's work on a crate called `some-lib`.

### File Setup

***main.rs***:
```rust,no_run
#[macro_use]
extern crate papyrus;

use papyrus::prelude::*;

# #[cfg(not(feature = "runnable"))]
# fn main() {}

# #[cfg(feature = "runnable")]
fn main() {
  let mut repl = repl!();

  repl.run(&mut ());
}
```

***lib.rs***:
```rust
pub struct MyStruct {
  pub a: i32,
  pub b: i32,
}

impl MyStruct {
  pub fn new(a: i32, b: i32) -> Self {
    MyStruct { a, b }
  }

  pub fn add_contents(&self) -> i32 {
    self.a + self.b
  }
}
```

***Cargo.toml***:
```toml
[package]
name = "some-lib"

...

[lib]
name = "some_lib"
crate-type = ["rlib" ]
path = "src/lib.rs" # you may need path to the library

[dependencies]
papyrus = { version = "*", crate-type = [ "rlib" ] }
...
```

Notice that you will have to specify the _library_ with a certain `crate-type`.
Papyrus links using an `rlib` file, but it is shown that you can also build multiple library files.
If you build this project you should find a `libsome_lib.rlib` sitting in your build directory.
Papyrus uses this to link when compiling.
The `papyrus` dependency also requires a `crate-type` specification.
If not specified, references to `papyrus` in the _library_ will cause compilation errors when
running the REPL.

### REPL

Run this project (`cargo run`). It should spool up fine and prompt you with `papyrus=>`.
Now you can try to use the linked crate.

```sh
papyrus=> some_lib::MyStruct::new(20, 30).add_contents()
papyrus [out0]: 50
```

## Behind the scenes

- Papyrus takes the crate name you specify and will add this as `extern crate CRATE_NAME;` to the source file.
- When setting the external crate name, the `rlib` library is found and copied into the compilation directory.
  - Papyrus uses `std::env::current_exe()` to find the executing folder, and searches for the `rlib` file in that folder (`libCRATE_NAME.rlib`)
  - Specify the path to the `rlib` library if it is located in a different folder
- When compiling the REPL code, a rustc flag is set, linking the `rlib` such that `extern crate CRATE_NAME;` works.

## Passing `MyStruct` data through

Keep the example before, but alter the `main.rs` file.

***main.rs***:
```rust,ignore
#[macro_use]
extern crate papyrus;
extern crate some_lib;

use some_lib::MyStruct;

# #[cfg(not(feature = "runnable"))]
# fn main() {}

# #[cfg(feature = "runnable")]
fn main() {
  let mut app_data = MyStruct::new(20, 10);

  let mut repl = repl!(some_lib::MyStruct);

  repl.data = repl
    .data
    .with_extern_crate("some_lib", None)
    .expect("failed creating repl data");

  repl.run(&mut app_data);
}
```

Run this project (`cargo run`).
It should spool up fine and prompt you with `papyrus=>`.
Now you can try to use the linked data.
The linked data is in a variable `app_data`. It is borrowed or mutably borrowed depending on the
REPL state.

```sh
papyrus=> app_data.add_contents()
papyrus [out0]: 50
```

## Notes
### Panics

To avoid crashing the application on a panic, `catch_unwind` is employed.
This function requires data that crosses the boundary be `UnwindSafe`, making `&` and `&mut` not valid data types.
Papyrus uses `AssertUnwindSafe` wrappers to make this work, however it makes `app_data` vulnerable to breaking
invariant states if a panic is triggered.

The developer should keep this in mind when implementing a linked REPL. 
Some guidelines:

1. Keep the app_data that is being transfered simple.
2. Develop wrappers that only pass through a _clone_ of the data.

