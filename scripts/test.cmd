cd test-resources/external_crate
cargo build
cd ../..
cargo test -- --test-threads=1