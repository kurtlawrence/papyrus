# Changelog

## 0.6.1

- Papyrus now formats the source code written to file.
- Help messages have been colourized for clarity.

## 0.6.0

`papyrus` now works with stable rust! 🎉

```sh
rustup default stable
cargo install papyrus
```

## 0.5.2

- Added the `.cancel` and `.c` commands which allow users to cancel out of the current input. This lets you exit more input loops if a leading closing bracket was defined.

## 0.5.1

- Turned off colouring for Windows, not yet working as intended.

## 0.5.0

- Added a version query to `crates.io`. When the repl is run interactively it will check the version number and print if there is a later version available.
- Added `query()` method to query `papyrus` version on `crates.io`.

## 0.4.2

- Added in benchmarks.

## 0.4.1

- Updated `.travis.yml` to initiate code coverage.

## 0.4.0

- First pass refactoring towards library stablisation.
- Added `ExternCrate` support. You can now use `extern crate crate_name as alias;`. This will work in most cases, please raise a PR if it doesn't.
- Compilation status is redirected to the console.
- Panics now get shown, and statements won't be added if code panics.