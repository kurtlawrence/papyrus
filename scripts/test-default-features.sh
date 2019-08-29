# Assume working directory is project root
set -e

cd papyrus
cargo build
cargo test -- --test-threads=1
cd ..
