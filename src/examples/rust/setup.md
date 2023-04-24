# Setup

Before getting into the more theoretical parts of grammars and APIs, let's first
make sure we're all set up.

## Rust and Cargo

The easiest way to install Rust and Cargo together is to follow the instructions
on [rustup.rs](https://rustup.rs). Once that is out of the way, make sure you
add *pest* to your `Cargo.toml`:

```toml
pest = "^1.0"
pest_derive = "^1.0"
```

*pest_derive* is the part of the parser that analyzes, verifies, optimizes, and
generates the code that then makes use of the APIs found in the *pest* crate.
This is separate because the actual procedural macro that derives the parser for
you is linked at compile time.

## The `.pest` grammar file

The actual grammar gets saved in separate `.pest` files, relative to Cargo's
`src` directory. They are then used in order to derive an implementation of the
[Parser][1] trait.

Due to the fact that procedural macro do not offer an API to tell the compiler
which files are relevant to compilation, it is necessary to provide a small hint
in the form of a debug-only `const` in order to make sure that your grammar gets
recompiled after every change.

So, you should add the following code to the Rust file where you want the parser
to be.

[1]: https://docs.rs/pest/1.0/pest/trait.Parser.html

```rust
// Don't forget to request use of the pest and pest_derive crate
use pest::Parser;
use pest_derive::Parser

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("path/to/rust.pest"); // relative to this file

#[derive(Parser)]
#[grammar = "path/to/rust.pest"] // relative to src
struct RustParser;
```
