# Somen
![status](https://img.shields.io/badge/status-Active-brightgreen?style=flat-square)
[![crates.io](https://img.shields.io/crates/v/somen?style=flat-square)](https://crates.io/crates/somen)
[![Downloads](https://img.shields.io/crates/d/somen?style=flat-square)](https://crates.io/crates/somen)
[![Downloads (latest)](https://img.shields.io/crates/dv/somen?style=flat-square)](https://crates.io/crates/somen)
[![License](https://img.shields.io/crates/l/somen?style=flat-square)](https://github.com/watcol/somen/blob/main/LICENSE)
![Lint](https://img.shields.io/github/workflow/status/watcol/somen/Lint?label=lint&style=flat-square)
![Test](https://img.shields.io/github/workflow/status/watcol/somen/Test?label=test&style=flat-square)

Somen is an asynchronous LL(k) parser combinator.

## Usage
Add to your `Cargo.toml`:
```toml
[dependencies]
somen = "0.3.0"
```

If you are in the `no_std` environment:
```toml
[dependencies.somen]
version = "0.3.0"
default-features = false
features = ["alloc"]   # If you have an allocator implementation
```

See [examples](https://github.com/watcol/somen/blob/main/examples) for more usage.

## Documentation
API Documentations are available on [here](https://docs.rs/somen).

## License
This program is licensed under the MIT license.
See [LICENSE](https://github.com/watcol/somen/blob/main/LICENSE) for details.
