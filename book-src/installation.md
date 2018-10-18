# Installation

`papyrus` depends on `proc-macro2` and `syn` which contains features that are only available on a nightly compiler. Further to this, the features are underneath a config flag, so compiling requires the `RUSTFLAGS` environment variable to include `--cfg procmacro2_semver_exempt`.

Linux, Mac

```bash
RUSTFLAGS="--cfg procmacro2_semver_exempt" cargo install papyrus
```

Windows

```bash
$env:RUSTFLAGS="--cfg procmacro2_semver_exempt"
cargo install papyrus;
````