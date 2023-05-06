# Operator precedence

There are several methods for dealing with operator precedence in `pest`:

1. directly in the PEG grammar;
2. using a `PrecClimber` (*deprecated*);
3. using a `PrattParser`.

Given `PrattParser` is the most general available method that supports
unary prefix and suffix operators, we provide more details on its usage.

## Pratt Parser
The following pest grammar defines a calculator which can be used for Pratt parsing.

```pest
WHITESPACE   =  _{ " " | "\t" | NEWLINE }
  
program      =   { SOI ~ expr ~ EOI }
  expr       =   { prefix? ~ primary ~ postfix? ~ (infix ~ prefix? ~ primary ~ postfix? )* }
    infix    =  _{ add | sub | mul | div | pow }
      add    =   { "+" } // Addition
      sub    =   { "-" } // Subtraction
      mul    =   { "*" } // Multiplication
      div    =   { "/" } // Division
      pow    =   { "^" } // Exponentiation
    prefix   =  _{ neg }
      neg    =   { "-" } // Negation
    postfix  =  _{ fac }
      fac    =   { "!" } // Factorial
    primary  =  _{ int | "(" ~ expr ~ ")" }
      int    =  @{ (ASCII_NONZERO_DIGIT ~ ASCII_DIGIT+ | ASCII_DIGIT) }
```

Below is a `PrattParser` that is able to parse an expr in the above grammar. The order of precedence corresponds to the order in which `op` is called. Thus, `mul` will have higher precedence than `add`. Operators can also be chained with `|` to give them equal precedence.

```rust
let pratt =
    PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::infix(Rule::pow, Assoc::Right))
        .op(Op::postfix(Rule::fac))
        .op(Op::prefix(Rule::neg));
```

To parse an expression, call the `map_primary`, `map_prefix`, `map_postfix`, `map_infix` and parse methods as follows:

```rust
fn parse_expr(pairs: Pairs<Rule>, pratt: &PrattParser<Rule>) -> i32 {
    pratt
        .map_primary(|primary| match primary.as_rule() {
            Rule::int  => primary.as_str().parse().unwrap(),
            Rule::expr => parse_expr(primary.into_inner(), pratt), // from "(" ~ expr ~ ")"
            _          => unreachable!(),
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::neg  => -rhs,
            _          => unreachable!(),
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::fac  => (1..lhs+1).product(),
            _          => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| match op.as_rule() {
            Rule::add  => lhs + rhs,
            Rule::sub  => lhs - rhs,
            Rule::mul  => lhs * rhs,
            Rule::div  => lhs / rhs,
            Rule::pow  => (1..rhs+1).map(|_| lhs).product(),
            _          => unreachable!(),
        })
        .parse(pairs)
}
```

Note that `map_prefix`, `map_postfix` and `map_infix` only need to be specified if the grammar contains the corresponding operators.
