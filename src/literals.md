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

This is because every one of the rules you define will populate and `enum` name
`Rule`. Thus, if any rules conflicts with Rust's naming scheme, it will error
out with an ambiguous message which is why *pest* tries its best to catch any
possible error before it reaches the compiler.

A simple (but less elegant) solution here would be to suffix these rules with
`_lit`:

```
true_lit  = { "true" }
false_lit = { "false" }
bool  = { true_lit | false_lit }
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
