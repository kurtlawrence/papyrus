# Assume working directory is project root
set -e

cd test-resources/external_crate
cargo build
cd ../..
