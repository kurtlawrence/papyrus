set -e
cargo modoc
cd papyrus && cargo build && cd ..
mdbook build docs
mdbook test -L papyrus/target/debug,papyrus/target/debug/deps docs
