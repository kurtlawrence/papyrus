# Assume working directory is project root
set -e

cd papyrus
cargo build --no-default-features
cargo test --no-default-features -- --test-threads=1
cd ..
