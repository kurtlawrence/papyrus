# File Interaction

`papyrus` comes with the command `load` which lets you load a file as source code. `papyrus` accepts both `*.rs` and `*.rscript` files, and handles the code slightly differently when it loads it into the `Repl`.

- `*.rs` files are treated as source files where code is defined with functions and structs, etc.
- `*.rscript` files are treated like expressions, where each line is passed through to the `main` function.

Load is used with the syntax `=>.load <filename>`.

## Examples

### `.rs` File

Let us define a source file `pwr.rs`.

```rust
fn pwr(base: u32, exponent: u32) -> u32 {
  (0..=exponent).into_iter().fold(1, |acc, x| acc * base)
}
```

In the papyrus repl:

```terminal
papyrus=> .load pwr.rs
papyrus=> pwr(2,3)
papyrus [out0]: 8
papyrus=>
```

### `.rscript` File

Let us define a source file `count_files.rscript`.

```rust
let dir = std::env::current_dir().unwrap();
let mut count = 0;
for entry in dir.read_dir().unwrap() {
  if entry.is_ok() {
    count += 1;
  }
}
count
```

In the papyrus repl:

```terminal
papyrus=> .load count_files.rscript
papyrus=> pwr(2,3)
papyrus [out0]: 8
papyrus=>
```