# Literals

A good place to start when writing out the grammar of a language are the
literals. For our small Rust subset, the literals that we are going to define
are booleans, integers, floating point numbers, strings, types, and identifiers.

## Booleans

Defining booleans is probably the easiest step. We need a rule with two
variants, `true` and `false`:

```
bool = { "true" | "false" }
```

This, however, will only generate a token for the `bool` rule without telling us
which variant it is, forcing us to dig through the input in order to see whether
it is `true` or `false`. In order to parse this only once and get the necessary
information right away, we can make `true` and `false` separate rules:

```
true  = { "true" }
false = { "false" }
bool  = { true | false }
```

Unfortunately, running `cargo check` will print the following error:

```
grammar error

 --> rust.pest:1:1
  |
1 | true  = { "true" }
  | ^--^
  |
  = true is a rust keyword

grammar error

 --> rust.pest:2:1
  |
2 | false = { "false" }
  | ^---^
  |
  = false is a rust keyword
```

This is because every one of the rules you define will populate an `enum` named
`Rule`. Thus, if any rules conflict with Rust's naming scheme, it will error
out with an ambiguous message which is why *pest* tries its best to catch any
possible error before it reaches the compiler.

A simple (but less elegant) solution here would be to suffix these rules with
`_lit`:

```
true_lit  = { "true" }
false_lit = { "false" }
bool      = { true_lit | false_lit }
```

This seems to work fine, but before we head on to integers, let's first write a
couple of tests. *pest* comes with a handy macro for asserting parse results
named [parses_to!][1].

```rust
#[test]
fn true_lit() {
    parses_to! {
        parser: RustParser,     // our parser struct
        input: "true",          // the input we're testing
        rule: Rule::bool,       // the rule that should be run
        tokens: [
            bool(0, 4, [        // name_of_rule(start_pos, end_pos, [children])
                true_lit(0, 4)  // name_of_rule(start_pos, end_pos): no children
            ])
        ]
    };
}

#[test]
fn false_lit() {
    parses_to! {
        parser: RustParser,
        input: "false",
        rule: Rule::bool,
        tokens: [
            bool(0, 5, [
                false_lit(0, 5)
            ])
        ]
    };
}
```

[1]: https://docs.rs/pest/1.0.0-beta/pest/macro.parses_to.html

## Integers

Although not as trivial as the booleans, integers should be quite
straightforward. In our implementation, we will only implement decimal integers
which start with a digit, then continue with any mixture of digits and
underscores:

```
int = { '0'..'9' ~ ('0'..'9' | "_")* }
```

In the example above, the range defining a digit (`'0'..'9'`) is repeated and
can be turned into a rule. Since we do not want it to generate tokens or be
reported in errors, we will make it silent (`_`).

```
digit = _{ '0'..'9' }
int   =  { digit ~ (digit | "_")* }
```

Testing a few cases like `"0"`, `"01"`, `"0___"`, `"1_000_000"` should suffice.

## Floating point numbers

Here is where it starts to become a little bit tricky. Floating points come in
two different shapes:

* integer literal followed by a `'.'`, followed by another optional integer
  literal, followed by an optional exponent
* integer literal, followed by a an exponent

By abstracting the definition of the exponent, the grammar will look like this:

```
float = {
    int ~ "." ~ int? ~ exp? |
    int ~ exp
}
```

The exponent part is a case insensitive `'e'`, followed by an optional sign
(`'+'`/`'-'`), followed by an integer. To match a string insensitively, you can
use the `^` prefix operator. Again, we would like to keep track of the signs in
order not to have to parse again, so we make the signs separate rules:

```
plus  = { "+" }
minus = { "-" }
exp   = { ^"e" ~ (plus | minus)? ~ int }
```

Testing floating point numbers should take into consideration their nested
integer and exponent tokens:

```rust
#[test]
fn zero_point() {
    parses_to! {
        parser: RustParser,
        input: "0.",
        rule: Rule::float,
        tokens: [
            float(0, 2, [
                int(0, 1)
            ])
        ]
    };
}

#[test]
fn one_exponent() {
    parses_to! {
        parser: RustParser,
        input: "1e10",
        rule: Rule::float,
        tokens: [
            float(0, 4, [
                int(0, 1),
                exp(1, 4, [
                    int(2, 4)
                ])
            ])
        ]
    };
}
```

More interesting test cases could be `"0.e0"`, `"0.0e+0"`, `"0.0"`,
`"0__.0__e-0__"`.
