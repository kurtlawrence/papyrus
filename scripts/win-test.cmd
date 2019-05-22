cd test-resources/external_crate
cargo build
cd ../..
cd papyrus
cargo test -- --test-threads=1