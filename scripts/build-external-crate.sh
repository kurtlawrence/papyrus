set -e
# Build external_crate
cd test-resources/external_crate
cargo build
cd ../..
# Run tests
cargo test --all-features