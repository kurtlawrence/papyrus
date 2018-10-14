[![Build Status](https://travis-ci.com/kurtlawrence/papyrus.svg?branch=master)](https://travis-ci.com/kurtlawrence/papyrus)

# papyrus

Current version: `0.2.2`

## A rust script running tool.

See the [docs.](https://docs.rs/papyrus/)
Look at progress and contribute on [github.](https://github.com/kurtlawrence/papyrus)

Install `papyrus`.

```bash
cargo install papyrus
```

Add right click context menu. (May need admin rights)

```bash
papyrus rc-add
```

Remove right click context menu. (May need admin rights)

```bash
papyrus rc-remove
```

Run papyrus from command line.

```bash
papyrus run path_to_src_file.rs
papyrus run path_to_script_file.rscript
```

## Implementation Notes

- Right click on a `.rs` or `.rscript` file and choose `Run with Papyrus` to compile and run code!
- Papyrus will take the contents of the source code and construct a directory to be used with `cargo`. For now the directory is created under a `.papyrus` directory in the users home directory.
- The compiled binary will be executed with the current directory the one that houses the file. So `env::current_dir()` will return the directory of the `.rs` or `.rscript` file.

## Example - .rs

File `hello.rs`.

```
extern crate some_crate;

fn main() {
  println!("Hello, world!");
}
```

Use papyrus to execute code.

```bash
papyrus run hello.rs
```

The `src/main.rs` will be populated with the same contents as `hello.rs`. A `Cargo.toml` file will be created, where `some_crate` will be added as a dependency `some-crate = "*"`.

## Example - .rscript

File `hello.rscript`.

```
extern crate some_crate;

println!("Hello, world!");
```

Use papyrus to execute code.

```bash
papyrus run hello.rscript
```

The `src/main.rs` will be populated with a main function encapsulating the code, and crate references placed above it. A similar `Cargo.toml` will be created as before.

License: MIT
