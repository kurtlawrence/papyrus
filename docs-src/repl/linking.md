# Linking

---

When running a repl you might want to link an external crate. The specific use case is a developer wants to link the crate they are working on into the repl for the user to be able to use.

`papyrus` has this functionality but makes some assumptions that the developer will need to be aware of.

## Worked Example

---

Let's work on a crate called `some-lib`.

### File Setup

***main.rs***:

```rust
use papyrus::{Repl, ReplData};

fn main() {
  let mut data = ReplData::default()
    .with_external_crate("some_lib", None)
    .expect("failed linking crate");
  let repl = Repl::default_terminal(&mut data);

  repl.run();
}
```

***lib.rs***:

```rust
pub struct MyStruct {
  a: u32,
  b: i32,
}

impl MyStruct {
  pub fn new(a: u32, b: i32) -> Self {
  MyStruct { a, b }
  }

  pub fn add_contents(&self) -> i32 {
    let c = self.a as i32;
    c + self.b
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
crate-type = ["rlib", "staticlib"]
path = "src/lib.rs" # you may need path to the library

[dependencies]
papyrus = "*"
...
```

Notice that you will have to specify the library with a certain `crate-type`. `papyrus` links using an `rlib` file, but I have shown that you can also build multiple library files.

If you build this project you should find a `libsome_lib.rlib` sitting in your build directory. `papyrus` uses this to link when compiling.

### Repl

Run this project (`cargo run`). It should spool up fine and prompt you with `papyrus=>`. Now you can try to use the linked crate.

```sh
papyrus=> some_lib::MyStruct::new(20, 30).add_contents()
papyrus [out0]: 50
```

## What's going on?

---

- `papyrus` takes the crate name you specify and will add this as `extern crate CRATE_NAME;` to the source file.
- When setting the external crate name, the `rlib` library is found and copied into the compilation directory.
  - `papyrus` uses `std::env::current_exe()` to find the executing folder, and searches for the `rlib` file in that folder (`libCRATE_NAME.rlib`)
  - Specify the path to the `rlib` library if it is located in a different folder
- When compiling the repl code, a rust flag is set, linking the `rlib` such that `extern crate CRATE_NAME;` works.