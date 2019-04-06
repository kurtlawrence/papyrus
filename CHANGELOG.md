# Changelog

## 0.8.1

- Re-export `azul` crate in `prelude`.
- Re-export `linefeed::memory::MemoryTerminal` struct in `prelude`.
- Added `CommandResult::Empty`
- Updated `cmdtree` to `v0.5` so users can output to a writer.
- Added `CommandResult::ActionOnAppData` so user can take actions on app data
- `CommandResult` actions pass through writer.

## 0.8.0

- The macros have been changed. Now `repl!` and `repl_with_term!` are used to create a `Repl` instance, with access to the public `.data` field to customise the repl data.
- there is no more variants for borrowing patterns, only the specified data type
- Added a `push_input()` function a `Read` repl. This allows for pushing of characters onto the repl without reading from `stdin`.
- `Repl` now takes ownership of `ReplData`. It did not make sense to borrow as you would have to drop `Repl` to make any changes to `ReplData`...
- Added `push_input_str()` - an extension of `push_input()`
- Added `eval_async` functions for `Repl`s in the `Evaluate` stage
- `widgets::pad` is now released!

## 0.7.0

Major change in api as the library is shifted towards a new repl direction.

- `Repl`s are now state machines
- Cannot construct a `Repl` using a file
- Cannot `evaluate` off a `Repl` (this has been altered to work with state machine)
- `Command` now requires to return a repl in print state (`Result<Repl<Print>, ()>`)
- `Repl`s use `ReplData` which is passed between states and instantiated outside the repl machine
- Documentation is not up to date, and will not be until some features stablise
- `Repl`s can now link external crates and data.
- Removed context menu functionality
- Removed repl file loading

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