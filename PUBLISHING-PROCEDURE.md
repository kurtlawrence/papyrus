# Publishing Procedure

1. `cargo my-readme` - updates the readmes on the `lib.rs` and `main.rs`
2. `cargo test` (use the no threading version)
3. Increment version number
4. Update `changelog.md` in `docs`
5. `mdbook build` - build the docs html
6. Commit and push changes in `auto` branch. If successful, travis will merge into master and publish