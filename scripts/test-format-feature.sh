# Assume working directory is project root
set -e

cd papyrus
cargo build --no-default-features --features="format"
cargo test --no-default-features --features="format" -- --test-threads=1
cd ..
