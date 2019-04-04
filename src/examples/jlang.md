# Example: the J language

The [J language] is an array programming language influenced by APL. Arrays are multi-dimensional; each has an integer *rank* specifying the number of its dimensions, in addition to a single-dimension array *shape* specifying the count of each of those dimensions.

Operations on arrays (and scalars) are referred to as *verbs*; verbs can be modified by *adverbs* and composed together as higher-order functions. Verbs are either monadic (taking a single argument or an array of arguments to its right) or dyadic (taking two arguments, one on either side).

Here's an example of a J program:

```j
'Some array operations...'
*: 1 2 3 4
matrix =: 2 3 $ 5 + 2 3 4 5 6 7
10 * matrix
1 + 10 20 30
1 2 3 + 10
residues =: 2 | 0 1 2 3 4 5 6 7
residues
```

Using a J [compiler] or [interpreter] to compile/run the above program yields the following on standard out:

```
Some array operations...
1 4 9 16
 70  80  90
100 110 120
11 21 31
11 12 13
0 1 0 1 0 1 0 1
```

In this section we'll write a grammar for a subset of J. We'll then walk through a parser that builds an AST by iterating over the rules that pest gives us.

## The Grammar

We'll build up a pest grammar section by section, starting with the program rule:

```pest
program = _{ SOI ~ "\n"* ~ (stmt ~ "\n"+) * ~ stmt? ~ EOI }
```

Each J program contains statements delimited by one or more newlines. Notice the leading underscore, which tells pest to [silence] the `program` rule -- we don't want `program` to appear as a token in the parse stream, we want the underlying statements instead.

A statement is simply an expression, and since there's only one such possibility, we also [silence] this `stmt` rule as well, and thus our parser will receive an iterator of underlying `expr`s from pest:

```pest
stmt = _{ expr }
```

An expression could be an assignment to a variable identifier, a monadic expression, a dyadic expression, a single string, or an array of terms:

```pest
expr = { assgmtExpr | monadicExpr | dyadicExpr | string | terms }
```

A monadic expression is an action with a right operand, a dyadic expression is an action with both left and right operands, and assignments associate identifiers with expressions:

```pest
monadicExpr = { action ~ expr }
dyadicExpr = { (monadicExpr | terms) ~ action ~ expr }
assgmtExpr = { ident ~ "=:" ~ expr }
```

A list of terms should contain at least one decimal, integer, identifier, or parenthesized expression; we care only about those underlying values, so we make the `term` rule [silent] with a leading underscore:

```pest
terms = { term+ }
term = _{ decimal | integer | ident | "(" ~ expr ~ ")" }
```

Verbs can be modified by adverbs; in this grammar that notion is encapsulated in the `action` rule. A few of J's verbs, and one of J's adverbs, is defined in this grammar; J's [full vocabulary] is much more extensive.

```pest
action = { verb ~ adverb* }
verb = { ">:" | "*:" | "-" | "%" | "#" | ">." |
          "+" | "*" | "<"  | "=" | "^" | "|" | ">" | "$" }
adverb = { "/" }
```

Now we can get into lexing rules. Numbers in J are represented as usual, with the exception that negatives are represented using a leading `_` underscore (because `-` is a verb that performs negation as a monad and subtraction as a dyad).  Identifiers in J must start with a letter, but can contain numbers thereafter. Strings are surrounded by single quotes; quotes themselves can be embedded by escaping them with an additional quote.

Notice how we use pest's `@` modifier to require *[atomic]ity* for each of these rules, meaning [implicit whitespace] is forbidden, and that interior rules (i.e., `ASCII_ALPHA` in `ident`) become [silent] -- when our parser receives any of these tokens from pest, they will be terminal:

```pest
integer = @{ "_"? ~ ASCII_DIGIT+ }
decimal = @{ "_"? ~ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT* }
ident = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }
string = @{ "'" ~ ( "''" | (!"'" ~ ANY) )* ~ "'" }
```

Whitespace in J consists solely of spaces and tabs; newlines are significant (they delimit statements) and so they are excluded from this rule:

```pest
WHITESPACE = _{ " " | "\t" }
```

Finally, we must handle comments. Comments in J start with `NB.` and continue to the end of the line on which they are found. Critically, we must not consume the newline at the end of the comment line; this is needed to separate any statement that might precede the comment from the statement on the succeeding line. We also omit [the `EOI` marker] from consumption here for the same reason as it relates to parsing the program as a whole:

```pest
COMMENT = _{ "NB." ~ ( !("\n" | EOI) ~ ANY)* }
```

## Parsing and AST Generation

This section will walk through a parser that uses the pest grammar above. Library includes and self-explanatory code are omitted here; you can find the parser in its entirety at `examples/jlang-parser/src/main.rs` of this book's repository. 

First we'll enumerate the verbs defined in our grammar, distinguishing between monadic and dyadic verbs. These enumerations will be be used as labels in our AST:

```rust
pub enum MonadicVerb {
    Increment = 1,
    Square = 2,
    Negate = 3,
    Reciprocal = 4,
    Tally = 5,
    Ceiling = 6,
    ShapeOf = 7,
}

pub enum DyadicVerb {
    Plus = 1,
    Times = 2,
    LessThan = 3,
    LargerThan = 4,
    Equal = 5,
    Minus = 6,
    Divide = 7,
    Power = 8,
    Residue = 9,
    Copy = 10,
    LargerOf = 11,
    LargerOrEqual = 12,
    Shape = 13,
}
```

Then we'll enumerate the various kinds of AST nodes:

```rust
pub enum AstNode {
    Print(Box<AstNode>),
    Integer(i32),
    DoublePrecisionFloat(f64),
    MonadicOp { verb: MonadicVerb, expr: Box<AstNode> },
    DyadicOp { verb: DyadicVerb, lhs: Box<AstNode>, rhs: Box<AstNode>},
    Terms(Vec<AstNode>),
    Reduce { verb: DyadicVerb, expr: Box<AstNode> },
    IsGlobal{ident: String, expr: Box<AstNode>},
    Ident(String),
    Str(CString),
}
```

To parse top-level statements in a J program, we have the following `parse` function that accepts a J program in string form and passes it to pest for parsing. We get back a sequence of pairs from pest. As specified in the grammar, a statement can only consist of an expression, so the `match` below parses each of those top-level expressions and wraps them in a `Print` AST node in keeping with the J interpreter's REPL behavior:

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

AST nodes are built from expressions by walking pest's pair iterator in lockstep with the expectations set out in our grammar file. Common behaviors are abstracted out into separate functions, such as `parse_monadic_action` and `parse_dyadic_action`, and pairs representing expressions themselves are passed in recursive calls to `build_ast_from_expr`:

```rust
fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> AstNode {

    match pair.as_rule() {
        Rule::expr => build_ast_from_expr(pair.into_inner().next().unwrap()),
        Rule::monadicExpr => {
            let mut pair = pair.into_inner();
            let action = pair.next().unwrap();
            let expr = pair.next().unwrap();
            let expr = build_ast_from_expr(expr);
            parse_monadic_action(action, expr)
        },
        Rule::dyadicExpr => {
            let mut pair = pair.into_inner();
            let lhspair = pair.next().unwrap();
            let lhs = build_ast_from_expr(lhspair);
            let action = pair.next().unwrap();
            let rhspair = pair.next().unwrap();
            let rhs = build_ast_from_expr(rhspair);
            parse_dyadic_action(action, lhs, rhs)
        },
        Rule::terms => {
            let terms : Vec<AstNode> = pair.into_inner()
                .map(build_ast_from_term)
                .collect();
            // If there's just a single term, return it without
            // wrapping it in a Terms node.
            match terms.len() {
                1 => terms.get(0).unwrap().clone(),
                _ => Terms(terms),
            }
        },
        Rule::assgmtExpr => {
            let mut pair = pair.into_inner();
            let ident = pair.next().unwrap();
            let expr = pair.next().unwrap();
            let expr = build_ast_from_expr(expr);
            AstNode::IsGlobal { ident : String::from(ident.as_str()),
                expr : Box::new(expr) }
        },
        Rule::string => {
            let str = &pair.as_str();
            // Strip leading and ending quotes.
            let str = &str[1..str.len() - 1];
            // Escaped string quotes become single quotes here.
            let str = str.replace("''", "'");
            AstNode::Str(CString::new(&str[..]).unwrap())
        }
        unknown_expr => panic!("Unexpected expression: {:?}", unknown_expr),
    }
}
```

Dyadic verbs are mapped from their string representations to AST nodes in a straightforward way:

```rust
fn parse_dyadic_action(pair : pest::iterators::Pair<Rule>,
                       lhs : AstNode,
                       rhs : AstNode) -> AstNode {
    let mut pair = pair.into_inner();
    let verb = pair.next().unwrap();
    let adverbs : Vec<pest::iterators::Pair<_>> = pair.collect();

    // Adverbs not currently supported on dyadic verbs.
    assert_eq!(adverbs.len(), 0);

    let lhs = Box::new(lhs);
    let rhs = Box::new(rhs);

    match verb.as_str() {
        "+" => AstNode::DyadicOp { verb: DyadicVerb::Plus, lhs, rhs },
        "*" => AstNode::DyadicOp { verb: DyadicVerb::Times, lhs, rhs },
        "-" => AstNode::DyadicOp { verb: DyadicVerb::Minus, lhs, rhs },
        "<" => AstNode::DyadicOp { verb: DyadicVerb::LessThan, lhs, rhs },
        "=" => AstNode::DyadicOp { verb: DyadicVerb::Equal, lhs, rhs },
        ">" => AstNode::DyadicOp { verb: DyadicVerb::LargerThan, lhs, rhs },
        "%" => AstNode::DyadicOp { verb: DyadicVerb::Divide, lhs, rhs },
        "^" => AstNode::DyadicOp { verb: DyadicVerb::Power, lhs, rhs },
        "|" => AstNode::DyadicOp { verb: DyadicVerb::Residue, lhs, rhs },
        "#" => AstNode::DyadicOp { verb: DyadicVerb::Copy, lhs, rhs },
        ">." => AstNode::DyadicOp { verb: DyadicVerb::LargerOf, lhs, rhs },
        ">:" => AstNode::DyadicOp { verb: DyadicVerb::LargerOrEqual, lhs, rhs },
        "$" => AstNode::DyadicOp { verb: DyadicVerb::Shape, lhs, rhs },
        _ => panic!("Unexpected dyadic verb: {}", verb)
    }
}
```

As are monadic verbs:

```rust
fn parse_monadic_action(pair : pest::iterators::Pair<Rule>,
                        expr : AstNode) -> AstNode {
    let mut pair = pair.into_inner();
    let verb = pair.next().unwrap();
    let adverbs : Vec<pest::iterators::Pair<_>> = pair.collect();

    match verb.as_str() {
        ">:" => {
            assert_eq!(adverbs.len(), 0);
            AstNode::MonadicOp { verb: MonadicVerb::Increment,
                expr: Box::new(expr) }
        },
        "*:" => {
            assert_eq!(adverbs.len(), 0);
            AstNode::MonadicOp { verb: MonadicVerb::Square,
                expr: Box::new(expr) }
        },
        "-" => {
            match adverbs.len() {
                0 => AstNode::MonadicOp { verb: MonadicVerb::Negate,
                    expr: Box::new(expr) },
                1 => AstNode::Reduce { verb: DyadicVerb::Minus,
                    expr: Box::new(expr) },
                _ => panic!("Unsupported number of adverbs for '-': {}", adverbs.len())
            }
        },
        "%" => {
            assert_eq!(adverbs.len(), 0);
            AstNode::MonadicOp { verb: MonadicVerb::Reciprocal,
                expr: Box::new(expr) }
        },
        "#" => {
            assert_eq!(adverbs.len(), 0);
            AstNode::MonadicOp { verb: MonadicVerb::Tally,
                expr: Box::new(expr) }
        },
        ">." => {
            match adverbs.len() {
                0 => AstNode::MonadicOp { verb: MonadicVerb::Ceiling,
                    expr: Box::new(expr) },
                1 => AstNode::Reduce { verb: DyadicVerb::LargerOf,
                    expr: Box::new(expr) },
                _ => panic!("Unsupported number of adverbs for '>.': {}", adverbs.len())
            }
        },
        "+" => {
            assert_eq!(adverbs.len(), 1);
            assert_eq!(adverbs[0].as_str(), "/");
            AstNode::Reduce { verb: DyadicVerb::Plus,
                expr: Box::new(expr) }
        },
        "*" => {
            assert_eq!(adverbs.len(), 1);
            assert_eq!(adverbs[0].as_str(), "/");
            AstNode::Reduce { verb: DyadicVerb::Times,
                expr: Box::new(expr) }
        },
        "$" => {
            assert_eq!(adverbs.len(), 0);
            AstNode::MonadicOp { verb: MonadicVerb::ShapeOf,
                expr: Box::new(expr) }
        },
        _ => panic!("Unsupported monadic action verb: {}", verb.as_str()),
    }
}
```

Finally, we define a function to process terms such as numbers and strings. Numbers require some manuevering to handle J's leading underscores representing negation, but other than that the process is typical:

```rust
fn build_ast_from_term(pair: pest::iterators::Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::integer => {
            let istr = pair.as_str();
            let (sign, istr) = match &istr[..1] {
                "_" => (-1, &istr[1..]),
                _ => (1, &istr[..]),
            };
            let integer : i32 = istr.parse().unwrap();
            AstNode::Integer(sign * integer)
        },
        Rule::decimal => {
            let dstr = pair.as_str();
            let (sign, dstr) = match &dstr[..1] {
                "_" => (-1.0, &dstr[1..]),
                _ => (1.0, &dstr[..]),
            };
            let mut flt : f64 = dstr.parse().unwrap();
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

We can now define a `main` function to pass J programs to our pest-enabled parser:

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
+ / 100 200 300
```

We'll get the following abstract syntax tree on stdout when we run the parser:

```shell
$ cargo run
  [ ... ]
[Print(DyadicOp { verb: Power, lhs: DoublePrecisionFloat(-2.5), rhs: Integer(3) }), Print(MonadicOp { verb: Square, expr: DoublePrecisionFloat(4.8) }), Print(IsGlobal { ident: "title", expr: Str("Spinning at the Boundary") }), Print(MonadicOp { verb: Square, expr: Terms([Integer(-1), Integer(2), Integer(-3), Integer(4)]) }), Print(DyadicOp { verb: Plus, lhs: Terms([Integer(1), Integer(2), Integer(3)]), rhs: Terms([Integer(10), Integer(20), Integer(30)]) }), Print(DyadicOp { verb: Plus, lhs: Integer(1), rhs: Terms([Integer(10), Integer(20), Integer(30)]) }), Print(DyadicOp { verb: Plus, lhs: Terms([Integer(1), Integer(2), Integer(3)]), rhs: Integer(10) }), Print(DyadicOp { verb: Residue, lhs: Integer(2), rhs: Terms([Integer(0), Integer(1), Integer(2), Integer(3), Integer(4), Integer(5), Integer(6), Integer(7)]) }), Print(IsGlobal { ident: "another", expr: Str("It\'s Escaped") }), Print(DyadicOp { verb: Residue, lhs: Integer(3), rhs: Terms([Integer(0), Integer(1), Integer(2), Integer(3), Integer(4), Integer(5), Integer(6), Integer(7)]) }), Print(DyadicOp { verb: Times, lhs: DyadicOp { verb: Plus, lhs: Integer(2), rhs: Integer(1) }, rhs: DyadicOp { verb: Plus, lhs: Integer(2), rhs: Integer(2) } }), Print(DyadicOp { verb: Times, lhs: Integer(3), rhs: DyadicOp { verb: Plus, lhs: Integer(2), rhs: Integer(1) } }), Print(DyadicOp { verb: Plus, lhs: Integer(1), rhs: DyadicOp { verb: Divide, lhs: Integer(3), rhs: Integer(4) } }), Print(IsGlobal { ident: "x", expr: Integer(100) }), Print(DyadicOp { verb: Minus, lhs: Ident("x"), rhs: Integer(1) }), Print(IsGlobal { ident: "y", expr: DyadicOp { verb: Minus, lhs: Ident("x"), rhs: Integer(1) } }), Print(Ident("y")), Print(Reduce { verb: Plus, expr: Terms([Integer(100), Integer(200), Integer(300)]) })]
```

[J language]: https://jsoftware.com/
[interpreter]: https://jsoftware.com/stable.htm
[compiler]: https://github.com/mattjquinn/jcompiler
[full vocabulary]: https://code.jsoftware.com/wiki/NuVoc
[implicit whitespace]: ../grammars/syntax.md#implicit-whitespace
[atomic]: ../grammars/syntax.md#atomic
[silence]: ../grammars/syntax.md#silent-and-atomic-rules
[silent]: ../grammars/syntax.md#silent-and-atomic-rules
[the `EOI` marker]: ../grammars/syntax.md#start-and-end-of-input
