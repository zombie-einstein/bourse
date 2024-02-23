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

See [here](examples/) for simulation examples and
[the docs](https://zombie-einstein.github.io/bourse/pages/example.html)
for an annotated example.

## Rust

### Getting Started

The library consists of two core crates:

- `bourse-book` a library implementing a limit order
  book. Full documentation can be found
  [here](https://docs.rs/bourse-book/latest/bourse_book/).

- `bourse-de` a discrete-event market simulation library.
  Full documentation can be found
  [here](https://docs.rs/bourse-de/latest/bourse_de/)

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
