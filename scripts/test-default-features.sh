set -e
./scripts/build-external-crate.sh
cargo test -- --test-threads=1
