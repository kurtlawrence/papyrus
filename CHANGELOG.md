# Changelog

## 0.17.0
- Path to examples in README fixed
- REPL `kserd` has `format` feature enabled
- Libraries are `Box`ed to avoid possible segmentation faults
- Added an on exit function to `RunCallbacks`.
- Terminal interface now has history that can be retrieved using Up and Down arrows
- `:static-files` supports glob patterns.
- Added `:mod clear` that clears previous REPL inputs

## 0.17.1
- Fix compiling bug with latest `crossbeam_channel` release

## 0.17.2
- Fix compiling bug with latest `syn` release

## 0.16.0
- Increase `libloading` dependency to `0.6`.
- Increase `kserd` dependency to `0.4`.
- Move MSRV to `1.42` to support `libloading`.

## 0.15.0
- Added ability to have static files, through API and REPL interface.

## 0.14.0
- Fix completion overwriting lines before bug #59
- Add `persistent_module_code` on the `LinkingConfiguration`, this is used to solve bug #57
- **Breaking Change:** `AppDataAction` now expects closure that has access to `&mut ReplData` - #60
- **Breaking Change:** Add functionality to inject callbacks when running a REPL - #58
- Output `.dylib` on MacOS - #66
- Update to `kserd` 0.3
- **Breaking Change:** Renaming of `to_*` to `into_*` in various places.
- **Breaking Change:** Removed `SourceCode::new` in-lieu of `Default` impl.
- **Breaking Change:** `format()` no _does not replace new lines with spaces_.
- Rework of interface. Increased testability and robustness.
- **Breaking Change:** Increase MSRV to `1.39`

### 0.14.1
- Fix compilation issue with `backtrace` dependency

## 0.13.0
- Restructure of repository
- Fix unintended indenting on unix when using `println!` macro
- Fix drawing issues when input would overflow into other lines
- Libraries can now be cached and not dropped once evaluation is finished.
- Removed ability to redirect output of evaluation.
  - This functionality is broken and requires 
    a. More testing
    b. Better use case
- Fixed the high cpu usage regression
- Increase requirement of MSRV to 1.36

### 0.13.1
- Up the event waiting duration from 1 millisecond to 5

## 0.12.0
- Handle `Item::Use` when parsing input.
- Use of `backtrace` crate to dump complete backtrace with panic handling.
- Update `crossterm` to 0.13.
- Remove `rustfmt-nightly` from dependencies, using `rustfmt` binary to format code
  - This was done as building on nightly crashed too often.
- Handle inner attribute syntax such as `#![feature(test)]`.
- Changed `Item` from `String` to `(String, bool)`.
- Remove `mortal` dependency, moving to `crossterm` for input. Fixes erratic typing input and not
    showing on windows.
- Bug fixes

### 0.12.1
- Handle `Item::Macro`.
- Updated linking documentation.
- Code improvements
- Terminal interface input events are buffered so multiline stdin is handled correctly.
- Update `crossterm` dependency to 0.14.

## 0.11.0

- docs: Add `cmds` module documentation.
- docs: Add `complete` module documentation.
- Move `code` and `linking` module to root level.

## 0.10.0

- Removed `Repl.run_with_completion()`, use `Repl.run()` with `racer-completion` feature instead.
- docs: Update output module documentation.
- New `runnable` interface. Foundational change to allow for more flexiblity and growth.
- Switch command prefix `.` to `:`.
- Add `Repl.run_async()` function that accepts an `Arc<Mutex<D>>` for data.
- Change `eval_async` to accept `Arc<Mutex<D>>`.

## 0.9.0

- mod paths now complete
- No longer need a terminal interface, maintains own internal buffer for output
- Fixed bug where using as library would not compile.

## 0.8.1

- Re-export `azul` crate in `prelude`.
- Re-export `linefeed::memory::MemoryTerminal` struct in `prelude`.
- Re-export `cmdtree` items into the `prelude`.
- Added `CommandResult::Empty`
- Updated `cmdtree` to `v0.5` so users can output to a writer.
- Added `CommandResult::ActionOnAppData` so user can take actions on app data
- `CommandResult` actions pass through writer.
- async repls require `RwLock` rather than `Mutex`.
- Added module support

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
