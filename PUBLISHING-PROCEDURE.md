# Publishing Procedure

1. `cargo my-readme` - updates the readmes on the `lib.rs` and `main.rs`
2. `cargo outdated` - check outdated versions
3. `cargo update` - updates the compatible versions, best to update major versions if possible
4. `cargo test` (use the no threading version)
5. `cargo bench` - see if there are any regressions
6. Increment version number
7. Update `changelog.md` in `docs-src`
8. `mdbook build` - build the docs html
9. Commit and push changes in `auto` branch. If successful, travis will merge into master and publish