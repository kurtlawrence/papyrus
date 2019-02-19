
**Calling crate**

needs in `Cargo.toml`

```toml
[lib]
name = "PKG_NAME"
crate-type = ["rlib", "staticlib"]
path = "src/lib.rs" # you may need path to the library
```

where `PKG_NAME` is the name of the crate library