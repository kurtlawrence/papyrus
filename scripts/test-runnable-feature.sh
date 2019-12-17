# Assume working directory is project root
set -e

cd papyrus
cargo build --no-default-features --features="runnable"
cargo test --no-default-features --features="runnable" -- --test-threads=1
cd ..
