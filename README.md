<img src="https://raw.githubusercontent.com/ackwell/ironworks/main/logo.png" alt="ironworks" align="right" height="250">

# ironworks

Modular FFXIV data toolkit written in rust.

[![Crates.io](https://img.shields.io/crates/v/ironworks?style=flat-square)](https://crates.io/crates/)
[![docs.rs](https://img.shields.io/docsrs/ironworks?style=flat-square)](https://docs.rs/ironworks)

ironworks is pre-1.0, and as such its API should be considered unstable. Breaking API changes will be published on new minor versions.

---

To minimise unused code & dependencies, ironworks is split into a number of discrete features. No features are enabled by default - pick the ones you want to use!

| Feature  | Description                                                |
| -------- | ---------------------------------------------------------- |
| `excel`  | Read data from Excel databases.                            |
| `ffxiv`  | Bindings for using ironworks with FFXIV.                   |
| `sqpack` | Navigate and extract files from the SqPack package format. |

## Getting started

```toml
[dependencies]
ironworks = {version = "0.2.0", features = ["excel", "ffxiv", "sqpack"]}
```

```rust
use ironworks::{
  Error,
  excel::Excel,
  ffxiv,
  sqpack::SqPack,
};

fn main() -> Result<(), Error> {
  // Read out files directly.
  let sqpack = SqPack::new(ffxiv::FsResource::search().unwrap());
  let file = sqpack.file("exd/root.exl")?;

  // Read fields out of excel.
  let excel = Excel::new(ffxiv::SqPackResource::new(&sqpack));
  let field = excel.sheet("Item")?.row(37362)?.field(0)?;

  Ok(())
}
```

## Using generated sheets from Excel

In addition to reading individual fields as shown above, it's possible to entire rows at a time into a struct. To faciliate this, generated sheet definitions are available as a git dependency.

**Warning:** The data used to generate these structs does not provide any stability guarantees whatsoever. As such, any update to sheet structs should be considered as a semver-major update.

```toml
[dependencies]
# ...
ironworks_sheets = {git = "https://github.com/ackwell/ironworks", branch = "sheets/saint-coinach"}
```

```rust
// ...
use ironworks_sheets::{for_type, sheet};

fn main() -> Result<(), Error> {
  // ...
  let field = excel.sheet(for_type::<sheet::Item>())?.row(37362)?.singular;
  // ...
}
```
