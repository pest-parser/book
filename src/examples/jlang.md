# Example: The J language

The J language is an array programming language influenced by APL.
In J, operations on individual numbers (`2 * 3`) can just as easily 
be applied to entire lists of numbers (`2 * 3 4 5`, returning `6 8 10`).

Operators in J are referred to as *verbs*.
Verbs are either *monadic* (taking a single argument, such as `*: 3`, "3 squared")
or *dyadic* (taking two arguments, one on either side, such as `5 - 4`, "5 minus 4").

Here's an example of a J program:

```j
'A string'

*: 1 2 3 4

matrix =: 2 3 $ 5 + 2 3 4 5 6 7
10 * matrix

1 + 10 20 30
1 2 3 + 10

residues =: 2 | 0 1 2 3 4 5 6 7
residues
```

Using J's [interpreter] to run the above program
yields the following on standard out:

```
A string

1 4 9 16

 70  80  90
100 110 120

11 21 31
11 12 13

0 1 0 1 0 1 0 1
```

In this section we'll write a grammar for a subset of J. We'll then walk 
through a parser that builds an AST by iterating over the rules that 
`pest` gives us. You can find the full source code
[within this book's repository].

## The Grammar

We'll build up a grammar section by section, starting with
the program rule:

```pest
program = _{ SOI ~ "\n"* ~ (stmt ~ "\n"+) * ~ stmt? ~ EOI }
```

Each J program contains statements delimited by one or more newlines.
Notice the leading underscore, which tells `pest` to [silence] the `program`
rule &mdash; we don't want `program` to appear as a token in the parse stream,
we want the underlying statements instead.

A statement is simply an expression, and since there's only one such 
possibility, we also [silence] this `stmt` rule as well, and thus our 
parser will receive an iterator of underlying `expr`s:

```pest
stmt = _{ expr }
```

An expression can be an assignment to a variable identifier, a monadic
expression, a dyadic expression, a single string, or an array of terms:

```pest
expr = {
      assgmtExpr
    | monadicExpr
    | dyadicExpr
    | string
    | terms
}
```

A monadic expression consists of a verb with its sole operand on the right;
a dyadic expression has operands on either side of the verb.
Assignment expressions associate identifiers with expressions.

In J, there is no operator precedence &mdash; evaluation is right-associative
(proceeding from right to left), with parenthesized expressions evaluated
first.

```pest
monadicExpr = { verb ~ expr }

dyadicExpr = { (monadicExpr | terms) ~ verb ~ expr }

assgmtExpr = { ident ~ "=:" ~ expr }
```

A list of terms should contain at least one decimal, integer, 
identifier, or parenthesized expression; we care only about those 
underlying values, so we make the `term` rule [silent] with a leading 
underscore:

```pest
terms = { term+ }

term = _{ decimal | integer | ident | "(" ~ expr ~ ")" }
```

A few of J's verbs are defined in this grammar;
J's [full vocabulary] is much more extensive.

```pest
verb = {
    ">:" | "*:" | "-"  | "%" | "#" | ">."
  | "+"  | "*"  | "<"  | "=" | "^" | "|"
  | ">"  | "$"
}
```

Now we can get into lexing rules. Numbers in J are represented as 
usual, with the exception that negatives are represented using a 
leading `_` underscore (because `-` is a verb that performs negation 
as a monad and subtraction as a dyad).  Identifiers in J must start 
with a letter, but can contain numbers thereafter. Strings are 
surrounded by single quotes; quotes themselves can be embedded by 
escaping them with an additional quote.

Notice how we use `pest`'s `@` modifier to make each of these rules [atomic],
meaning [implicit whitespace] is forbidden, and
that interior rules (i.e., `ASCII_ALPHA` in `ident`) become [silent] &mdash;
when our parser receives any of these tokens from, they will be terminal:

```pest
integer = @{ "_"? ~ ASCII_DIGIT+ }

decimal = @{ "_"? ~ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT* }

ident = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

string = @{ "'" ~ ( "''" | (!"'" ~ ANY) )* ~ "'" }
```

Whitespace in J consists solely of spaces and tabs. Newlines are
significant because they delimit statements, so they are excluded
from this rule:

```pest
WHITESPACE = _{ " " | "\t" }
```

Finally, we must handle comments. Comments in J start with `NB.` and 
continue to the end of the line on which they are found. Critically, we must 
not consume the newline at the end of the comment line; this is needed 
to separate any statement that might precede the comment from the statement 
on the succeeding line.

```pest
COMMENT = _{ "NB." ~ ( !"\n" ~ ANY)* }
```

## Parsing and AST Generation

This section will walk through a parser that uses the grammar above.
Library includes and self-explanatory code are omitted here; you can find 
the parser in its entirety [within this book's repository].

First we'll enumerate the verbs defined in our grammar, distinguishing between 
monadic and dyadic verbs. These enumerations will be be used as labels 
in our AST:

```rust
pub enum MonadicVerb {
    Increment,
    Square,
    Negate,
    Reciprocal,
    Tally,
    Ceiling,
    ShapeOf,
}

pub enum DyadicVerb {
    Plus,
    Times,
    LessThan,
    LargerThan,
    Equal,
    Minus,
    Divide,
    Power,
    Residue,
    Copy,
    LargerOf,
    LargerOrEqual,
    Shape,
}
```

Then we'll enumerate the various kinds of AST nodes:

```rust
pub enum AstNode {
    Print(Box<AstNode>),
    Integer(i32),
    DoublePrecisionFloat(f64),
    MonadicOp {
        verb: MonadicVerb,
        expr: Box<AstNode>,
    },
    DyadicOp {
        verb: DyadicVerb,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    Terms(Vec<AstNode>),
    IsGlobal {
        ident: String,
        expr: Box<AstNode>,
    },
    Ident(String),
    Str(CString),
}
```

To parse top-level statements in a J program, we have the following 
`parse` function that accepts a J program in string form and passes it 
to `pest` for parsing. We get back a sequence of [`Pair`]s. As specified
in the grammar, a statement can only consist of an expression, so the `match` 
below parses each of those top-level expressions and wraps them in a `Print` 
AST node in keeping with the J interpreter's REPL behavior:

```rust
pub fn parse(source: &str) -> Result<Vec<AstNode>, Error<Rule>> {
    let mut ast = vec![];

    let pairs = JParser::parse(Rule::program, source)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::expr => {
                ast.push(Print(Box::new(build_ast_from_expr(pair))));
            }
            _ => {}
        }
    }

    Ok(ast)
}
```

AST nodes are built from expressions by walking the [`Pair`] iterator in
lockstep with the expectations set out in our grammar file. Common behaviors 
are abstracted out into separate functions, such as `parse_monadic_verb`
and `parse_dyadic_verb`, and [`Pair`]s representing expressions themselves are
passed in recursive calls to `build_ast_from_expr`:

```rust
fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
        Rule::monadicExpr => {
            let mut pair = pair.into_inner();
            let verb = pair.next().unwrap();
            let expr = pair.next().unwrap();
            let expr = build_ast_from_expr(expr);
            parse_monadic_verb(verb, expr)
        }
        // ... other cases elided here ...
    }
}
```

Dyadic verbs are mapped from their string representations to AST nodes in 
a straightforward way:

```rust
fn parse_dyadic_verb(pair: pest::iterators::Pair<Rule>, lhs: AstNode, rhs: AstNode) -> AstNode {
    AstNode::DyadicOp {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        verb: match pair.as_str() {
            "+" => DyadicVerb::Plus,
            "*" => DyadicVerb::Times,
            "-" => DyadicVerb::Minus,
            "<" => DyadicVerb::LessThan,
            "=" => DyadicVerb::Equal,
            ">" => DyadicVerb::LargerThan,
            "%" => DyadicVerb::Divide,
            "^" => DyadicVerb::Power,
            "|" => DyadicVerb::Residue,
            "#" => DyadicVerb::Copy,
            ">." => DyadicVerb::LargerOf,
            ">:" => DyadicVerb::LargerOrEqual,
            "$" => DyadicVerb::Shape,
            _ => panic!("Unexpected dyadic verb: {}", pair.as_str()),
        },
    }
}
```

As are monadic verbs:

```rust
fn parse_monadic_verb(pair: pest::iterators::Pair<Rule>, expr: AstNode) -> AstNode {
    AstNode::MonadicOp {
        verb: match pair.as_str() {
            ">:" => MonadicVerb::Increment,
            "*:" => MonadicVerb::Square,
            "-" => MonadicVerb::Negate,
            "%" => MonadicVerb::Reciprocal,
            "#" => MonadicVerb::Tally,
            ">." => MonadicVerb::Ceiling,
            "$" => MonadicVerb::ShapeOf,
            _ => panic!("Unsupported monadic verb: {}", pair.as_str()),
        },
        expr: Box::new(expr),
    }
}
```

Finally, we define a function to process terms such as numbers and strings. 
Numbers require some manuevering to handle J's leading underscores 
representing negation, but other than that the process is typical:

```rust
fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::integer => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "_" => (-1, &istr[1..]),
                _ => (1, &istr[..]),
            };
            let integer: i32 = istr.parse().unwrap();
            AstNode::Integer(sign * integer)
        }
        Rule::decimal => {
            let dstr = pair.as_str();
            let (sign, dstr) = match &dstr[..1] {
                "_" => (-1.0, &dstr[1..]),
                _ => (1.0, &dstr[..]),
            };
            let mut flt: f64 = dstr.parse().unwrap();
            if flt != 0.0 {
                // Avoid negative zeroes; only multiply sign by nonzeroes.
                flt *= sign;
            }
            AstNode::DoublePrecisionFloat(flt)
        }
        Rule::expr => build_ast_from_expr(pair),
        Rule::ident => AstNode::Ident(String::from(pair.as_str())),
        unknown_term => panic!("Unexpected term: {:?}", unknown_term),
    }
}
```

## Running the Parser

We can now define a `main` function to pass J programs to our 
`pest`-enabled parser:

```rust
fn main() {
    let unparsed_file = std::fs::read_to_string("example.ijs")
      .expect("cannot read ijs file");
    let astnode = parse(&unparsed_file).expect("unsuccessful parse");
    println!("{:?}", &astnode);
}
```

Using this code in `example.ijs`:

```j
_2.5 ^ 3
*: 4.8
title =: 'Spinning at the Boundary'
*: _1 2 _3 4
1 2 3 + 10 20 30
1 + 10 20 30
1 2 3 + 10
2 | 0 1 2 3 4 5 6 7
another =: 'It''s Escaped'
3 | 0 1 2 3 4 5 6 7
(2+1)*(2+2)
3 * 2 + 1
1 + 3 % 4
x =: 100
x - 1
y =: x - 1
y
```

We'll get the following abstract syntax tree on stdout when we run 
the parser:

```shell
$ cargo run
  [ ... ]
[Print(DyadicOp { verb: Power, lhs: DoublePrecisionFloat(-2.5),
    rhs: Integer(3) }),
Print(MonadicOp { verb: Square, expr: DoublePrecisionFloat(4.8) }),
Print(IsGlobal { ident: "title", expr: Str("Spinning at the Boundary") }),
Print(MonadicOp { verb: Square, expr: Terms([Integer(-1), Integer(2),
    Integer(-3), Integer(4)]) }),
Print(DyadicOp { verb: Plus, lhs: Terms([Integer(1), Integer(2), Integer(3)]),
    rhs: Terms([Integer(10), Integer(20), Integer(30)]) }),
Print(DyadicOp { verb: Plus, lhs: Integer(1), rhs: Terms([Integer(10),
    Integer(20), Integer(30)]) }),
Print(DyadicOp { verb: Plus, lhs: Terms([Integer(1), Integer(2), Integer(3)]),
    rhs: Integer(10) }),
Print(DyadicOp { verb: Residue, lhs: Integer(2),
    rhs: Terms([Integer(0), Integer(1), Integer(2), Integer(3), Integer(4),
    Integer(5), Integer(6), Integer(7)]) }),
Print(IsGlobal { ident: "another", expr: Str("It\'s Escaped") }),
Print(DyadicOp { verb: Residue, lhs: Integer(3), rhs: Terms([Integer(0),
    Integer(1), Integer(2), Integer(3), Integer(4), Integer(5),
    Integer(6), Integer(7)]) }),
Print(DyadicOp { verb: Times, lhs: DyadicOp { verb: Plus, lhs: Integer(2),
    rhs: Integer(1) }, rhs: DyadicOp { verb: Plus, lhs: Integer(2),
        rhs: Integer(2) } }),
Print(DyadicOp { verb: Times, lhs: Integer(3), rhs: DyadicOp { verb: Plus,
    lhs: Integer(2), rhs: Integer(1) } }),
Print(DyadicOp { verb: Plus, lhs: Integer(1), rhs: DyadicOp { verb: Divide,
    lhs: Integer(3), rhs: Integer(4) } }),
Print(IsGlobal { ident: "x", expr: Integer(100) }),
Print(DyadicOp { verb: Minus, lhs: Ident("x"), rhs: Integer(1) }),
Print(IsGlobal { ident: "y", expr: DyadicOp { verb: Minus, lhs: Ident("x"),
    rhs: Integer(1) } }),
Print(Ident("y"))]
```

[J language]: https://jsoftware.com/
[interpreter]: https://jsoftware.com/
[full vocabulary]: https://code.jsoftware.com/wiki/NuVoc
[implicit whitespace]: ../grammars/syntax.md#implicit-whitespace
[atomic]: ../grammars/syntax.md#atomic
[silence]: ../grammars/syntax.md#silent-and-atomic-rules
[silent]: ../grammars/syntax.md#silent-and-atomic-rules
[`Pair`]: https://pest.rs/book/parser_api.html#pairs
[within this book's repository]: https://github.com/pest-parser/book/tree/master/examples/jlang-parser
