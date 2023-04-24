# Example: CSV

Comma-Separated Values is a very simple text format. CSV files consist of a
list of *records*, each on a separate line. Each record is a list of *fields*
separated by commas.

For example, here is a CSV file with numeric fields:

```
65279,1179403647,1463895090
3.1415927,2.7182817,1.618034
-40,-273.15
13,42
65537
```

Let's write a program that computes the **sum of these fields** and counts the
**number of records**.

## Setup

Start by initializing a new project using [Cargo]:

```shell
$ cargo init --bin csv-tool
     Created binary (application) project
$ cd csv-tool
```

Add the `pest` and `pest_derive` crates to the dependencies section in `Cargo.toml`:

```toml
[dependencies]
pest = "2.0"
pest_derive = "2.0"
```

## Writing the parser

`pest` works by compiling a description of a file format, called a *grammar*,
into Rust code. Let's write a grammar for a CSV file that contains numbers.
Create a new file named `src/csv.pest` with a single line:

```pest
field = { (ASCII_DIGIT | "." | "-")+ }
```

This is a description of every number field: each character is either an ASCII
digit `0` through `9`, a full stop `.`, or a hyphen&ndash;minus `-`. The plus
sign `+` indicates that the pattern can occur one or more times.

Rust needs to know to compile this file using `pest`:

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "csv.pest"]
pub struct CSVParser;
```

If you run `cargo doc`, you will see that `pest` has created the function
`CSVParser::parse` and an enum called `Rule` with a single variant
`Rule::field`.

Let's test it out! Rewrite `main`:

```rust
fn main() {
    let successful_parse = CSVParser::parse(Rule::field, "-273.15");
    println!("{:?}", successful_parse);

    let unsuccessful_parse = CSVParser::parse(Rule::field, "this is not a number");
    println!("{:?}", unsuccessful_parse);
}
```

```shell
$ cargo run
  [ ... ]
Ok([Pair { rule: field, span: Span { str: "-273.15", start: 0, end: 7 }, inner: [] }])
Err(Error { variant: ParsingError { positives: [field], negatives: [] }, location: Pos(0), path: None, line: "this is not a number", continued_line: None, start: (1, 1), end: None })
```

Yikes! That's a complicated type! But you can see that the successful parse was
`Ok`, while the failed parse was `Err`. We'll get into the details later.

For now, let's complete the grammar:

```pest
field = { (ASCII_DIGIT | "." | "-")+ }
record = { field ~ ("," ~ field)* }
file = { SOI ~ (record ~ ("\r\n" | "\n"))* ~ EOI }
```

The tilde `~` means "and then", so that `"abc" ~ "def"` matches `abc` followed
by `def`. (For this grammar, `"abc" ~ "def"` is exactly the same as `"abcdef"`,
although this is not true in general; see [a later chapter about
`WHITESPACE`].)

In addition to literal strings (`"\r\n"`) and built-ins (`ASCII_DIGIT`), rules
can contain other rules. For instance, a `record` is a `field`, and optionally
a comma `,` and then another `field` repeated as many times as necessary. The
asterisk `*` is just like the plus sign `+`, except the pattern is optional: it
can occur any number of times at all (zero or more).

There are two more rules that we haven't defined: `SOI` and `EOI` are two
special rules that match, respectively, the *start of input* and the *end of
input*. Without `EOI`, the `file` rule would gladly parse an invalid file! It
would just stop as soon as it found the first invalid character and report a
successful parse, possibly consisting of nothing at all!

## The main program loop

Now we're ready to finish the program. We will use [`File`] to read the CSV
file into memory. We'll also be messy and use [`expect`] everywhere.

```rust
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("numbers.csv").expect("cannot read file");

    // ...
}
```

Next we invoke the parser on the file. Don't worry about the specific types for
now. Just know that we're producing a [`pest::iterators::Pair`] that represents
the `file` rule in our grammar.

```rust
fn main() {
    // ...

    let file = CSVParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next().unwrap(); // get and unwrap the `file` rule; never fails

    // ...
}
```

Finally, we iterate over the `record`s and `field`s, while keeping track of the
count and sum, then print those numbers out.

```rust
fn main() {
    // ...

    let mut field_sum: f64 = 0.0;
    let mut record_count: u64 = 0;

    for record in file.into_inner() {
        match record.as_rule() {
            Rule::record => {
                record_count += 1;

                for field in record.into_inner() {
                    field_sum += field.as_str().parse::<f64>().unwrap();
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    println!("Sum of fields: {}", field_sum);
    println!("Number of records: {}", record_count);
}
```

If `p` is a parse result (a [`Pair`]) for a rule in the grammar, then
`p.into_inner()` returns an [iterator] over the named sub-rules of that rule.
For instance, since the `file` rule in our grammar has `record` as a sub-rule,
`file.into_inner()` returns an iterator over each parsed `record`. Similarly,
since a `record` contains `field` sub-rules, `record.into_inner()` returns an
iterator over each parsed `field`.

## Done

Try it out! Copy the sample CSV at the top of this chapter into a file called
`numbers.csv`, then run the program! You should see something like this:

```shell
$ cargo run
  [ ... ]
Sum of fields: 2643429302.327908
Number of records: 5
```

[Cargo]: https://doc.rust-lang.org/cargo/
[a later chapter about `WHITESPACE`]: ../grammars/syntax.html
[`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
[`expect`]: https://doc.rust-lang.org/std/option/enum.Option.html#method.expect
[`pest::iterators::Pair`]: https://docs.rs/pest/2.0/pest/iterators/struct.Pair.html
[`Pair`]: https://docs.rs/pest/2.0/pest/iterators/struct.Pair.html
[iterator]: https://doc.rust-lang.org/std/iter/index.html
