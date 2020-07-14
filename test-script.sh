set -e

# Build external_crate
cd test-resources/external_crate
cargo clean
cargo build
cd ../..
# Build external_kserd
cd test-resources/external_kserd
cargo clean
cargo build
cd ../..

# Update docs
cargo modoc
cargo +stable fmt

# Check formatting
cargo +stable fmt -- --check

# Check clippy
cargo clippy --all-features -- -D warnings 

# Run tests; includes interface testing
cargo test --all-features -- --test-threads=1

# Run cargo docs and ensure the linking is working
cargo doc

# To build docs, need clean target directory
# mdbook build docs
# mdbook test -L target/debug,target/debug/deps docs

echo "ALL TESTS PASSED"
