# Parser API

`pest` provides several ways of accessing the results of a successful parse.
The examples below use the following grammar:

```pest
number = { ASCII_DIGIT+ }                // one or more decimal digits
enclosed = { "(.." ~ number ~ "..)" }    // for instance, "(..6472..)"
sum = { number ~ " + " ~ number }        // for instance, "1362 + 12"
```

## Tokens

`pest` represents successful parses using *tokens*. Whenever a rule matches,
two tokens are produced: one at the *start* of the text that the rule matched,
and one at the *end*. For example, the rule `number` applied to the string
`"3130 abc"` would match and produce this pair of tokens:

```
"3130 abc"
 |   ^ end(number)
 ^ start(number)
```

Note that the rule doesn't match the entire input text. It only matches as much
text as possible, then stops if successful.

A token is like a cursor in the input string. It has a character position in
the string, as well as a reference to the rule that created it.

### Nested rules

If a named rule contains another named rule, tokens will be produced for *both*
rules. For instance, the rule `enclosed` applied to the string `"(..6472..)"`
would match and produce these four tokens:

```
"(..6472..)"
 |  |   |  ^ end(enclosed)
 |  |   ^ end(number)
 |  ^ start(number)
 ^ start(enclosed)
```

Sometimes, tokens might not occur at distinct character positions. For example,
when parsing the rule `sum`, the inner `number` rules share some start and end
positions:

```
"1773 + 1362"
 |   |  |   ^ end(sum)
 |   |  |   ^ end(number)
 |   |  ^ start(number)
 |   ^ end(number)
 ^ start(number)
 ^ start(sum)
```

In fact, for a rule that matches empty input, the start and end tokens will be
at the same position!

### Interface

Tokens are exposed as the [`Token`] enum, which has `Start` and `End` variants.
You can get an iterator of `Token`s by calling `tokens` on a parse result:

```rust
let parse_result = Parser::parse(Rule::sum, "1773 + 1362").unwrap();
let tokens = parse_result.tokens();

for token in tokens {
    println!("{:?}", token);
}
```

After a successful parse, tokens will occur as nested pairs of matching `Start`
and `End`. Both kinds of tokens have two fields:

- `rule`, which explains which rule generated them; and
- `pos`, which indicates their positions.

A start token's position is the first character that the rule matched. An end
token's position is the first character that the rule did not match &mdash;
that is, an end token refers to a position *after* the match. If a rule matched
the entire input string, the end token points to an imaginary position *after*
the string.

## Pairs

Tokens are not the most convenient interface, however. Usually you will want to
explore the parse tree by considering matching pairs of tokens. For this
purpose, `pest` provides the [`Pair`] type.

A `Pair` represents a matching pair of tokens, or, equivalently, the spanned
text that a named rule successfully matched. It is commonly used in several
ways:

- Determining which rule produced the `Pair`
- Using the `Pair` as a raw `&str`
- Inspecting the inner named sub-rules that produced the `Pair`

```rust
let pair = Parser::parse(Rule::enclosed, "(..6472..) and more text")
    .unwrap().next().unwrap();

assert_eq!(pair.as_rule(), Rule::enclosed);
assert_eq!(pair.as_str(), "(..6472..)");

let inner_rules = pair.into_inner();
println!("{}", inner_rules); // --> [number(3, 7)]
```

In general, a `Pair` might have any number of inner rules: zero, one, or more.
For maximum flexibility, `Pair::into_inner()` returns `Pairs`, which is an
iterator over each pair.

This means that you can use `for` loops on parse results, as well as iterator
methods such as `map`, `filter`, and `collect`.

```rust
let pairs = Parser::parse(Rule::sum, "1773 + 1362")
    .unwrap().next().unwrap()
    .into_inner();

let numbers = pairs
    .clone()
    .map(|pair| str::parse(pair.as_str()).unwrap())
    .collect::<Vec<i32>>();
assert_eq!(vec![1773, 1362], numbers);

for (found, expected) in pairs.zip(vec!["1773", "1362"]) {
    assert_eq!(Rule::number, found.as_rule());
    assert_eq!(expected, found.as_str());
}
```

`Pairs` iterators are also commonly used via the `next` method directly. If a
rule consists of a known number of sub-rules (for instance, the rule `sum` has
exactly two sub-rules), the sub-matches can be extracted with `next` and
`unwrap`:

```rust
let parse_result = Parser::parse(Rule::sum, "1773 + 1362")
    .unwrap().next().unwrap();
let mut inner_rules = parse_result.into_inner();

let match1 = inner_rules.next().unwrap();
let match2 = inner_rules.next().unwrap();

assert_eq!(match1.as_str(), "1773");
assert_eq!(match2.as_str(), "1362");
```

Sometimes rules will not have a known number of sub-rules, such as when a
sub-rule is repeated with an asterisk `*`:

```pest
list = { number* }
```

In cases like these it is not possible to call `.next().unwrap()`, because the
number of sub-rules depends on the input string &mdash; it cannot be known at
compile time.

## The `parse` method

A `pest`-derived [`Parser`] has a single method `parse` which returns a
`Result< Pairs, Error >`. To access the underlying parse tree, it is necessary
to `match` on or `unwrap` the result:

```rust
// check whether parse was successful
match Parser::parse(Rule::enclosed, "(..6472..)") {
    Ok(mut pairs) => {
        let enclosed = pairs.next().unwrap();
        // ...
    }
    Err(error) => {
        // ...
    }
}
```

Our examples so far have included the calls
`Parser::parse(...).unwrap().next().unwrap()`. The first `unwrap` turns the
result into a `Pairs`. If parsing had failed, the program would panic! We only
use `unwrap` in these examples because we already know that they will parse
successfully.

In the example above, in order to get to the `enclosed` rule inside of the
`Pairs`, we use the iterator interface. The `next()` call returns an
`Option<Pair>`, which we finally `unwrap` to get the `Pair` for the `enclosed`
rule.

### Using `Pair` and `Pairs` with a grammar

While the `Result` from `Parser::parse(...)` might very well be an error on
invalid input, `Pair` and `Pairs` often have more subtle behavior. For
instance, with this grammar:

```pest
number = { ASCII_DIGIT+ }
sum = { number ~ " + " ~ number }
```

this function will *never* panic:

```rust
fn process(pair: Pair<Rule>) -> f64 {
    match pair.as_rule() {
        Rule::number => str::parse(pair.as_str()).unwrap(),
        Rule::sum => {
            let mut pairs = pair.into_inner();

            let num1 = pairs.next().unwrap();
            let num2 = pairs.next().unwrap();

            process(num1) + process(num2)
        }
    }
}
```

`str::parse(...).unwrap()` is safe because the `number` rule only ever matches
digits, which `str::parse(...)` can handle. And `pairs.next().unwrap()` is safe
to call twice because a `sum` match *always* has two sub-matches, which is
guaranteed by the grammar.

Since these sorts of guarantees are awkward to express with Rust types, `pest`
only provides a few high-level types to represent parse trees. Nevertheless,
you *should* rely on the meaning of your grammar for properties such as
"contains *n* sub-rules", "is safe to `parse` to `f32`", and "never fails to
match". Idiomatic `pest` code uses `unwrap` and `unreachable!`.

## Spans and positions

Occasionally, you will want to refer to a matching rule in the context of the
raw source text, rather than the interior text alone. For example, you might
want to print the entire line that contained the match. For this you can use
[`Span`] and [`Position`].

A `Span` is returned from `Pair::as_span`. Spans have a start position and an
end position (which correspond to the start and end tokens of the rule that
made the pair).

Spans can be decomposed into their start and end `Position`s, which provide
useful methods for examining the string around that position. For example,
`Position::line_col()` finds out the line and column number of a position.

Essentially, a `Position` is a `Token` without a rule. In fact, you can use
pattern matching to turn a `Token` into its component `Rule` and `Position`.

[`Token`]: https://docs.rs/pest/2.0/pest/enum.Token.html
[`Pair`]: https://docs.rs/pest/2.0/pest/iterators/struct.Pair.html
[`Parser`]: https://docs.rs/pest/2.0/pest/trait.Parser.html
[`Span`]: https://docs.rs/pest/2.0/pest/struct.Span.html
[`Position`]: https://docs.rs/pest/2.0/pest/struct.Position.html
