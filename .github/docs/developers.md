# Developers

## Rust

The Rust API is broken down into 2 crates

- `./crates/order_book`: Simulated order-book
  and stock-market implementations.
- `./crates/step_sim`: A discrete event simulation
  library built around the order-book library.

The crates are organised under a single workspace
so cargo commands can be run from the repo root.

The rust [CI workflow](../workflows/pre_merge.yaml)
expects the following checks to pass

```
cargo test
cargo fmt --all -- --check
cargo clippy -- -Dwarnings
cargo doc --no-deps
```

and the examples to successfully run

```
cargo run --example random_agents
cargo run --example order_book
cargo run --example multi_asset
```

## Python

The Python API consists of

- `./rust`: A Rust-Python that provides Python classes
  forwarding functionality from the Rust simulation libraries.
- `./src`: Python library wrapping the Rust interface with
  additional simulation utilities.
- `./docs`: Sphinx documentation.
- `./examples`: Python simulation examples.

Bourse uses `hatch <https://hatch.pypa.io/latest/>`_ for
dependency management and `maturin <https://github.com/PyO3/maturin>`_
to build the combined Python-Rust package.

Development commands can be run from a virtual environment
using hatch

- `hatch run dev:develop`: Build and install the Python library
- `hatch run dev:lint`: Run Python linting
- `hatch run dev:test`: Run Python tests
- `hatch run dev:bench`: Run Python benchmarks
- `hatch run dev:build`: Build the Python library
- `hatch run dev:examples`: Run the Python examples

Documentation can be built and tested using
`hatch run docs:build` and `hatch run docs:test` respectively.
