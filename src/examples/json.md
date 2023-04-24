# Example: JSON

[JSON] is a popular format for data serialization that is derived from the
syntax of JavaScript. JSON documents are tree-like and potentially recursive
&mdash; two data types, *objects* and *arrays*, can contain other values,
including other objects and arrays.

Here is an example JSON document:

```json
{
    "nesting": { "inner object": {} },
    "an array": [1.5, true, null, 1e-6],
    "string with escaped double quotes" : "\"quick brown foxes\""
}
```

Let's write a program that **parses** the JSON to an Rust object, known as an
*abstract syntax tree*, then **serializes** the AST back to JSON.

## Setup

We'll start by defining the AST in Rust. Each JSON data type is represented by
an enum variant.

```rust
enum JSONValue<'a> {
    Object(Vec<(&'a str, JSONValue<'a>)>),
    Array(Vec<JSONValue<'a>>),
    String(&'a str),
    Number(f64),
    Boolean(bool),
    Null,
}
```

To avoid copying when deserializing strings, `JSONValue` borrows strings from
the original unparsed JSON. In order for this to work, we cannot interpret
string escape sequences: the input string `"\n"` will be represented by
`JSONValue::String("\\n")`, a Rust string with two characters, even though it
represents a JSON string with just one character.

Let's move on to the serializer. For the sake of clarity, it uses allocated
`String`s instead of providing an implementation of [`std::fmt::Display`],
which would be more idiomatic.

```rust
fn serialize_jsonvalue(val: &JSONValue) -> String {
    use JSONValue::*;

    match val {
        Object(o) => {
            let contents: Vec<_> = o
                .iter()
                .map(|(name, value)|
                     format!("\"{}\":{}", name, serialize_jsonvalue(value)))
                .collect();
            format!("{{{}}}", contents.join(","))
        }
        Array(a) => {
            let contents: Vec<_> = a.iter().map(serialize_jsonvalue).collect();
            format!("[{}]", contents.join(","))
        }
        String(s) => format!("\"{}\"", s),
        Number(n) => format!("{}", n),
        Boolean(b) => format!("{}", b),
        Null => format!("null"),
    }
}
```

Note that the function invokes itself recursively in the `Object` and `Array`
cases. This pattern appears throughout the parser. The AST creation function
iterates recursively through the parse result, and the grammar has rules which
include themselves.

## Writing the grammar

Let's begin with whitespace. JSON whitespace can appear anywhere, except inside
strings (where it must be parsed separately) and between digits in numbers
(where it is not allowed). This makes it a good fit for `pest`'s [implicit
whitespace]. In `src/json.pest`:

```pest
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
```

[The JSON specification] includes diagrams for parsing JSON strings. We can
write the grammar directly from that page. Let's write `object` as a sequence
of `pair`s separated by commas `,`.

```pest
object = {
    "{" ~ "}" |
    "{" ~ pair ~ ("," ~ pair)* ~ "}"
}
pair = { string ~ ":" ~ value }

array = {
    "[" ~ "]" |
    "[" ~ value ~ ("," ~ value)* ~ "]"
}
```

The `object` and `array` rules show how to parse a potentially empty list with
separators. There are two cases: one for an empty list, and one for a list with
at least one element. This is necessary because a trailing comma in an array,
such as in `[0, 1,]`, is illegal in JSON.

Now we can write `value`, which represents any single data type. We'll mimic
our AST by writing `boolean` and `null` as separate rules.

```pest
value = _{ object | array | string | number | boolean | null }

boolean = { "true" | "false" }

null = { "null" }
```

Let's separate the logic for strings into three parts. `char` is a rule
matching any logical character in the string, including any backslash escape
sequence. `inner` represents the contents of the string, without the
surrounding double quotes. `string` matches the inner contents of the string,
including the surrounding double quotes.

The `char` rule uses [the idiom `!(...) ~ ANY`], which matches any character
except the ones given in parentheses. In this case, any character is legal
inside a string, except for double quote `"` and backslash <code>\\</code>,
which require separate parsing logic.

```pest
string = ${ "\"" ~ inner ~ "\"" }
inner = @{ char* }
char = {
    !("\"" | "\\") ~ ANY
    | "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t")
    | "\\" ~ ("u" ~ ASCII_HEX_DIGIT{4})
}
```

Because `string` is marked [compound atomic], `string` [token pairs] will also
contain a single `inner` pair. Because `inner` is marked [atomic], no `char`
pairs will appear inside `inner`. Since these rules are atomic, no whitespace
is permitted between separate tokens.

Numbers have four logical parts: an optional sign, an integer part, an optional
fractional part, and an optional exponent. We'll mark `number` atomic so that
whitespace cannot appear between its parts.

```pest
number = @{
    "-"?
    ~ ("0" | ASCII_NONZERO_DIGIT ~ ASCII_DIGIT*)
    ~ ("." ~ ASCII_DIGIT*)?
    ~ (^"e" ~ ("+" | "-")? ~ ASCII_DIGIT+)?
}
```

We need a final rule to represent an entire JSON file. The only legal contents
of a JSON file is a single object or array. We'll mark this rule [silent], so
that a parsed JSON file contains only two token pairs: the parsed value itself,
and [the `EOI` rule].

```pest
json = _{ SOI ~ (object | array) ~ EOI }
```

## AST generation

Let's compile the grammar into Rust.

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "json.pest"]
struct JSONParser;
```

We'll write a function that handles both parsing and AST generation. Users of
the function can call it on an input string, then use the result returned as
either a `JSONValue` or a parse error.

```rust
use pest::error::Error;

fn parse_json_file(file: &str) -> Result<JSONValue, Error<Rule>> {
    let json = JSONParser::parse(Rule::json, file)?.next().unwrap();

    // ...
}
```

Now we need to handle `Pair`s recursively, depending on the rule. We know that
`json` is either an `object` or an `array`, but these values might contain an
`object` or an `array` themselves! The most logical way to handle this is to
write an auxiliary recursive function that parses a `Pair` into a `JSONValue`
directly.

```rust
fn parse_json_file(file: &str) -> Result<JSONValue, Error<Rule>> {
    // ...

    use pest::iterators::Pair;

    fn parse_value(pair: Pair<Rule>) -> JSONValue {
        match pair.as_rule() {
            Rule::object => JSONValue::Object(
                pair.into_inner()
                    .map(|pair| {
                        let mut inner_rules = pair.into_inner();
                        let name = inner_rules
                            .next()
                            .unwrap()
                            .into_inner()
                            .next()
                            .unwrap()
                            .as_str();
                        let value = parse_value(inner_rules.next().unwrap());
                        (name, value)
                    })
                    .collect(),
            ),
            Rule::array => JSONValue::Array(pair.into_inner().map(parse_value).collect()),
            Rule::string => JSONValue::String(pair.into_inner().next().unwrap().as_str()),
            Rule::number => JSONValue::Number(pair.as_str().parse().unwrap()),
            Rule::boolean => JSONValue::Boolean(pair.as_str().parse().unwrap()),
            Rule::null => JSONValue::Null,
            Rule::json
            | Rule::EOI
            | Rule::pair
            | Rule::value
            | Rule::inner
            | Rule::char
            | Rule::WHITESPACE => unreachable!(),
        }
    }

    // ...
}
```

The `object` and `array` cases deserve special attention. The contents of an
`array` token pair is just a sequence of `value`s. Since we're working with a
Rust iterator, we can simply map each value to its parsed AST node recursively,
then collect them into a `Vec`. For `object`s, the process is similar, except
the iterator is over `pair`s, from which we need to extract names and values
separately.

The `number` and `boolean` cases use Rust's `str::parse` method to convert the
parsed string to the appropriate Rust type. Every legal JSON number can be
parsed directly into a Rust floating point number!

We run `parse_value` on the parse result to finish the conversion.

```rust
fn parse_json_file(file: &str) -> Result<JSONValue, Error<Rule>> {
    // ...

    Ok(parse_value(json))
}
```

## Finishing

Our `main` function is now very simple. First, we read the JSON data from a
file named `data.json`. Next, we parse the file contents into a JSON AST.
Finally, we serialize the AST back into a string and print it.

```rust
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("data.json").expect("cannot read file");

    let json: JSONValue = parse_json_file(&unparsed_file).expect("unsuccessful parse");

    println!("{}", serialize_jsonvalue(&json));
}
```

Try it out! Copy the example document at the top of this chapter into
`data.json`, then run the program! You should see something like this:

```shell
$ cargo run
  [ ... ]
{"nesting":{"inner object":{}},"an array":[1.5,true,null,0.000001],"string with escaped double quotes":"\"quick brown foxes\""}
```

[JSON]: https://json.org/
[`std::fmt::Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
[implicit whitespace]: ../grammars/syntax.md#implicit-whitespace
[The JSON specification]: https://json.org/
[the idiom `!(...) ~ ANY`]: ../grammars/syntax.md#predicates
[compound atomic]: ../grammars/syntax.md#atomic
[token pairs]: ../parser_api.md#pairs
[atomic]: ../grammars/syntax.md#atomic
[silent]: ../grammars/syntax.md#silent-and-atomic-rules
[the `EOI` rule]: ../grammars/syntax.md#start-and-end-of-input
