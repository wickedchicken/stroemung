# stroemung

[![Crates.io](https://img.shields.io/crates/v/stroemung.svg)](https://crates.io/crates/stroemung)
[![CI](https://github.com/wickedchicken/stroemung/workflows/CI/badge.svg)](https://github.com/wickedchicken/stroemung/actions)
[![HireMe](hire_me.svg)](https://wickedchicken.github.io/post/hire-me/)

A Computational Fluid Dynamics (CFD) simulator in Rust

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install stroemung`

## Running

```sh
cargo run
```

## Generating test data

There is a Python script in the [`python/`](python/) directory which will generate
test data from the [NaSt2D][nast2d] program. See [`python/README.md`](python/README.md)
for more details.

## License

Licensed under the [MIT license](LICENSE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the MIT license, shall be
licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).

[nast2d]: https://ins.uni-bonn.de/content/software-nast2d
