# Introduction

*Speed or simplicity? Why not __both__?*

`pest` is a library for writing plain-text parsers in Rust.

Parsers that use `pest` are **easy to design and maintain** due to the use of
[Parsing Expression Grammars], or *PEGs*. And, because of Rust's zero-cost
abstractions, `pest` parsers can be **very fast**.

## Sample

Here is the complete grammar for a simple calculator [developed in a later chapter](examples/calculator.html):

```pest
num = @{ int ~ ("." ~ ASCII_DIGIT*)? ~ (^"e" ~ int)? }
int = { ("+" | "-")? ~ ASCII_DIGIT+ }

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
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;
        use Assoc::*;

        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
            .op(Op::infix(power, Right))
    };
}

fn eval(expression: Pairs<Rule>) -> f64 {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::num => primary.as_str().parse::<f64>().unwrap(),
            Rule::expr => eval(primary.into_inner()),
            _ => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add => lhs + rhs,
            Rule::subtract => lhs - rhs,
            Rule::multiply => lhs * rhs,
            Rule::divide => lhs / rhs,
            Rule::power => lhs.powf(rhs),
            _ => unreachable!(),
        })
        .parse(expression)
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
