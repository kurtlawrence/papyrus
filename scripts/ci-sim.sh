# Run in root dir
# sets rustup to stable, runs the few tests then does nightly with the full suite
set -e

rustup default 1.34.0
# remove this to avoid unknown compilation errors (needs clean directory)
rm -r -f papyrus/target/testing
./scripts/build-external-crate.sh
./scripts/test-no-features.sh
./scripts/test-runnable-feature.sh

rustup default stable
# remove this to avoid unknown compilation errors (needs clean directory)
rm -r -f papyrus/target/testing
./scripts/build-external-crate.sh
./scripts/test-no-features.sh
./scripts/test-runnable-feature.sh

rustup default nightly
# remove this to avoid unknown compilation errors (needs clean directory)
rm -r -f papyrus/target/testing
./scripts/build-external-crate.sh
./scripts/test-no-features.sh
./scripts/test-format-feature.sh
./scripts/test-racer-completion-feature.sh
./scripts/test-runnable-feature.sh
./scripts/test-default-features.sh

echo "All tests passed!"

