# Bourse

Rust market-simulator with Python API

## Python

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

- `bourse-book` an library implementing an market order book
- `bourse-de` a discrete-event market simulation library

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
