# Bourse

Rust market-simulator with a Python API

## Python

Full documentation can be found
[here](https://zombie-einstein.github.io/bourse/).

### Getting Started

Bourse can be installed via pip

```
pip install bourse
```

### Examples

See [here](examples/) for simulation examples.

## Rust

### Getting Started

The library consists of two core
crates:

- `bourse-book` a library implementing a market order
  book. Full documentation can be found
  [here](https://docs.rs/bourse-book/0.1.0/bourse_book/).

- `bourse-de` a discrete-event market simulation library.
  Full documentation can be found here
  [here](https://docs.rs/bourse-de/0.1.0/bourse_de/)

Both can be installed using cargo

```
cargo add bourse-book bourse-de
```

### Examples

Examples can be found in the relevant crates
[order book](crates/order_book/examples/) and
[simulation](crates/step_sim/examples/).

Examples can be run via cargo using

```
cargo run --example ...
```
