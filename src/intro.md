# Introduction

*Speed or simplicity? Why not __both__?*

`pest` is a library for writing plain-text parsers in Rust.

Parsers that use `pest` are **easy to design and maintain** due to the use of
[Parsing Expression Grammars], or *PEGs*. And, because of Rust's zero-cost
abstractions, `pest` parsers can be **very fast**.

## Sample

Here is the complete grammar for a simple calculator [developed in a (currently
unwritten) later chapter](examples/calculator.html):

```
num = @{ int ~ ("." ~ digit*)? ~ (^"e" ~ int)? }
    int = { ("+" | "-")? ~ digit+ }
    digit = { '0'..'9' }

operation = _{ add | subtract | multiply | divide | power }
    add      = { "+" }
    subtract = { "-" }
    multiply = { "*" }
    divide   = { "/" }
    power    = { "^" }

expr = { term ~ (operation ~ term)* }
term = _{ num | "(" ~ expr ~ ")" }

calculation = _{ SOI ~ expr ~ EOI }

WHITESPACE = _{ " " | "\t" }
```

And here is the function that uses that parser to calculate answers:

```rust
lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use Rule::*;
        use Assoc::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left),
            Operator::new(power, Right)
        ])
    };
}

fn eval(expression: Pairs<Rule>) -> f64 {
    PREC_CLIMBER.climb(
        expression,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::num => pair.as_str().parse::<f64>().unwrap(),
            Rule::expr => eval(pair.into_inner()),
            _ => unreachable!(),
        },
        |lhs: f64, op: Pair<Rule>, rhs: f64| match op.as_rule() {
            Rule::add      => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide   => lhs / rhs,
            Rule::power    => lhs.powf(rhs),
            _ => unreachable!(),
        },
    )
}
```

## About this book

This book provides an overview of `pest` as well as several example parsers.
For more details of `pest`'s API, check [the documentation].

Note that `pest` uses some advanced features of the Rust language. For an
introduction to Rust, consult the [official Rust book].

[Parsing Expression Grammars]: grammars/peg.html
[the documentation]: https://docs.rs/pest/
[official Rust book]: https://doc.rust-lang.org/stable/book/second-edition/
