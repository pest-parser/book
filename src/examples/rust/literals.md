# Literals

A good place to start when writing out the grammar of a language are the
literals. For our small Rust subset, the literals that we are going to define
are booleans, integers, floating point numbers, strings, characters, types, and
identifiers.

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

[1]: https://docs.rs/pest/1.0/pest/macro.parses_to.html

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

## Strings

Strings can get a little bit tricky since you have to make sure that you include
string escapes in your grammar. This is needed since you have no other way of
knowing exactly where the string ending quote will be and also because it makes
escaping easier later on.

Let's start by focusing on the high level definition. A string is a repetition
of raw string parts (containing no escapes) and actual escapes, all enclosed
within a pair of quotes:

```
string = { "\"" ~ (raw_string | escape)* ~ "\"" }
```

Raw strings can basically be any character apart from `'\'`, since that means
we're about to start an escape clause, and `'"'`, since that means we're about
to end the string. In order to match anything but these two characters, we look
ahead and fail the rule if we match these two characters. For this, we're going
to use a negative lookahead (`!`). After we made sure that we're matching the
correct character, we use the predefined rule `any` to actually force the parser
to skip this character, since the lookahead is non-destructive:

```
raw_string = { (!("\\" | "\"") ~ any)+ }
```

Rust string literals can be:

* predefined: `'\n'`, `'\r'`, `'\t'`, `'\\'`, `'\0'`,
* bytes: `'\x$$'`, where `$$` are two hexadecimal digits
* unicode: `\u{$}` - `\u{$$$$$$}`, where `$`s are from 1 up to 6 hexadecimal
  digits

A good place to start is to define the hex digit:

```
hex = _{ '0'..'9' | 'a'..'f' | 'A'..'F' }
```

To define a rule that can have from 1 up to 6 hex digits, pest offers a convenient
syntax `{m, n}`. Limits are inclusive. Note that `{n}`, `{n, }`, and `{, n}` syntaxes
exist too. Please see [non-terminals expressions][2] for more details.

```
unicode_hex = { hex{1, 6} }
```

[2]: https://docs.rs/pest_derive/1.0/pest_derive/#expressions

We now have everything we need to define escapes:

```
predefined = { "n" | "r" | "t" | "\\" | "0" | "\"" | "'" }
byte       = { "x" ~ hex{2} }
unicode    = { "u" ~ "{" ~ unicode_hex ~ "}" }
escape     = { "\\" ~ (predefined | byte | unicode) }
```

For the sake of compactness, we can write a single test that encompasses
everything interesting:

```rust
#[test]
fn string_with_all_escape_types() {
    parses_to! {
        parser: RustParser,
        input: r#""a\nb\x0Fc\u{a}d\u{AbAbAb}e""#,
        rule: Rule::string,
        tokens: [
            string(0, 28, [
                raw_string(1, 2),
                escape(2, 4, [
                    predefined(3, 4)
                ]),
                raw_string(4, 5),
                escape(5, 9, [
                    byte(6, 9)
                ]),
                raw_string(9, 10),
                escape(10, 15, [
                    unicode(11, 15, [
                        unicode_hex(13, 14)
                    ])
                ]),
                raw_string(15, 16),
                escape(16, 26, [
                    unicode(17, 26, [
                        unicode_hex(19, 25)
                    ])
                ]),
                raw_string(26, 27)
            ])
        ]
    };
}
```

## Characters

Characters are very similar to strings, with the obvious exception that may only
store one character:

```
chr = { "'" ~ (escape | any) ~ "'" }
```

Tests should cover at least the usual and the escape cases, e.g. `"'a'"`,
`"'\''"`.

## Types

Types should only be the few primitives defined here:

```
i32_ty  = { "i32" }
f32_ty  = { "f32" }
char_ty = { "char" }
str_ty  = { "str" }

ty = { i32_ty | f32_ty | char_ty | str_ty }
```

Writing one test for each of the four cases should suffice.

## Identifiers

Full-blown Rust identifiers can be a bit complex, so we will only focus on ASCII
variants:

* an identifier is made up of alphanumeric characters and underscores
* the first character cannot be a digit
* underscores need to be followed by at least another character

This can be implemented by having a choice clause between two cases:

```
ident_char = _{ 'a'..'z' | 'A'..'Z' | '0'..'9' | "_" }
ident      =  {
    ('a'..'z' | 'A'..'Z') ~ ident_char* |
    "_" ~ ident_char+
}
```

Interesting test cases could be `"aBc0"`, `"_0AbC"`.
