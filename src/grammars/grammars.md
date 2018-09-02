# Grammars

Like many parsing tools, `pest` operates using a *formal grammar* that is
distinct from your Rust code. The format that `pest` uses is called a *parsing
expression grammar*, or *PEG*. When building a project, `pest` automatically
compiles the PEG, located in a separate file, into a plain Rust function that
you can call.

## How to activate `pest`

Most projects will have at least two files that use `pest`: the parser (say,
`src/parser/mod.rs`) and the grammar (`src/parser/grammar.pest`). Assuming that
they are in the same directory:

```rust
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"] // relative to project `src`
struct MyParser;

const _GRAMMAR: &str = include_str!("grammar.pest"); // relative to this file
```

Whenever you compile this file, `pest` will automatically use the grammar file
to generate items like this:

```rust
pub enum Rules { /* ... */ }

impl Parser for MyParser {
    pub fn parse(Rules, &str) -> pest::Pairs { /* ... */ }
}
```

You will never see `enum Rules` or `impl Parser` as plain text! The code only
exists during compilation. However, you can use `Rules` just like any other
enum, and you can use `parse(...)` through the [`Pairs`] interface described in
the [Parser API chapter](../parser_api.html).

### Technical note: `_GRAMMAR`

Rust uses "incremental compilation" to avoid recompiling files that haven't
changed. Unfortunately, this means that changing the grammar file won't update
the generated parser, because the Rust source code hasn't changed!

To declare that the Rust source code depends on the grammar file, we copy that
grammar directly into an unused string constant using [`include_str!`].

## Warning about PEGs!

Parsing expression grammars look quite similar to other parsing tools you might
be used to, like regular expressions, BNF grammars, and others (Yacc/Bison,
LALR, CFG). However, PEGs behave subtly differently: PEGs are [eager],
[non-backtracking], [ordered], and [unambiguous].

Don't be scared if you don't recognize any of the above names! You're already a
step ahead of people who do &mdash; when you use `pest`'s PEGs, you won't be
tripped up by comparisons to other tools.

If you have used other parsing tools before, be sure to read the next section
carefully. We'll mention some common mistakes regarding PEGs.

[`Pairs`]: https://docs.rs/pest/2.0/pest/iterators/struct.Pairs.html
[`include_str!`]: https://doc.rust-lang.org/std/macro.include_str.html
[eager]: peg.html#eagerness
[non-backtracking]: peg.html#non-backtracking
[ordered]: peg.html#ordered-choice
[unambiguous]: peg.html#unambiguous
