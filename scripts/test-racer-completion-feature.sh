# Assume working directory is project root
set -e

cd papyrus
cargo build --no-default-features --features="racer-completion"
cargo test --no-default-features --features="racer-completion" -- --test-threads=1
cd ..
