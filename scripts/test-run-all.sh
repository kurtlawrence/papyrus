set -e

# remove this to avoid unknown compilation errors (needs clean directory)
rm -r -f papyrus/target/testing
./scripts/build-external-crate.sh
./scripts/test-no-features.sh
./scripts/test-format-feature.sh
./scripts/test-racer-completion-feature.sh
./scripts/test-runnable-feature.sh
./scripts/test-default-features.sh

echo "All tests passed!"

