cd ../azul
git pull
cd ../papyrus
git pull  # maybe git reset --hard master if Cargo.lock playing up
cd test-resources/external_crate
cargo build
cd ../..
cargo test -- --test-threads=1
cargo tarpaulin -v -l --out Html
mv tarpaulin-report.html /mnt/c/users/kurt/desktop/tarpaulin-report.html