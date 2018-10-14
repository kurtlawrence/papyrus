# Publishing Procedure

1. `cargo test`
2. Increment version number
3. `cargo readme > README.md`
4. Commit and push changes in `auto` branch. If successful, travis will merge into master and publish