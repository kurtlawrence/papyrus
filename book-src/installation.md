# Installation

Linux, Mac
```bash
RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo install papyrus
```

Windows
```bash
& { $env:RUSTFLAGS='--cfg procmacro2_semver_exempt'; cargo install papyrus; Remove-Item Env:\RUSTFLAGS }
```