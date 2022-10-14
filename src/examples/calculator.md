# Example: Calculator

This example focuses on the practical aspect of using a Pratt parser to parse expressions using `pest`.
To illustrate this, we build a parser for simple equations, and construct an abstract syntax tree.

## Precedence and associativity
In a simple equation multiplication and division are evaluated first, which means they have a higher precedence.
e.g. `1 + 2 * 3` is evaluated as `1 + (2 * 3)`, if the precedence was equal it would be `(1 + 2) * 3`.
For our system we have the following operands:
- highest precedence: multiplication & division
- lowest precedence: addition & subtraction

In the expression `1 + 2 - 3`, no operator is inherently more important than the other.
Addition, subtraction, multiplication and division are evaluated from left to right,
e.g. `1 - 2 + 3` is evaluated as `(1 - 2) + 3`. We call this property left associativity. 
Operators can also be right associative. For example, we usually evaluate the statement `x = y = 1` by first 
assigning `y = 1` and `x = 1` (or `x = y`) afterwards.

Associativity only matters if two operators have the same precedence, as is the case with addition and subtraction for 
example. This means that if we have an expression with only additions and subtractions, we can just evaluate it from 
left to right. `1 + 2 - 3` is equal to `(1 + 2) - 3`. And `1 - 2 + 3` is equal to `(1 - 2) + 3`.

To go from a flat list of operands separated by operators, it suffices to define a precedence and associativity for each 
operator. With these definitions an algorithm such as Pratt parsing is able to construct a corresponding 
expression tree.

If you are curious to know more about how Pratt parsing is implemented, Aleksey Kladov has a
[great tutorial](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) on implementing it
from scratch using Rust.

## Calculator example
We want our calculator to be able to parse simple equations that consist of integers and simple binary operators.
Additionally, we want to support parenthesis and unary minus.
For example:
```
1 + 2 * 3
-(2 + 5) * 16
```

## Grammar
We start by defining our atoms, bits of self-contained syntax that cannot be split up into smaller parts.
For our calculator we start with just simple integers:
```pest
// No whitespace allowed between digits
integer = @{ ASCII_DIGIT+ }

atom = _{ integer }
```

Next, our binary operators:
```pest
bin_op = _{ add | subtract | multiply | divide }
	add = { "+" }
	subtract = { "-" }
	multiply = { "*" }
	divide = { "/" }
```

These two rules will be the input to the
[`PrattParser`](https://docs.rs/pest/latest/pest/pratt_parser/struct.PrattParser.html). 
It expects to receive atoms separated by operators, like so: `atom, bin_op, atom, bin_op, atom, ...`.

Corresponding to this format, we define our rule for expressions:
```pest
expr = { atom ~ (bin_op ~ atom)* }
```
This defines the grammar which generates the required input for the Pratt parser.

## Abstract Syntax Tree
We want to convert our input into an abstract syntax tree.
For this we define the following types:

```rust
#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}
```

Note the `Box<Expr>` required because Rust 
[does not allow unboxed recursive types](https://doc.rust-lang.org/book/ch15-01-box.html#enabling-recursive-types-with-boxes). 

There is no separate atom type, any atom is also a valid expression.

## Pratt parser
The precedence of operations is defined in the Pratt parser.

An easy approach is to define the PrattParser as global using [`lazy_static`](https://docs.rs/lazy_static/1.4.0/lazy_static/).

Adhering to standard rules of arithmetic, 
we will define addition and subtraction to have lower priority than multiplication and division, 
and make all operators left associative.

```rust
lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
    };
}
```

We are almost there, the only thing that's left is to use our Pratt parser.
For this the `map_primary`, `map_infix`, and `parse` functions are used, the first two take functions and the third one takes an iterator over pairs.
`map_primary` is executed for every primary (atom), and `map_infix` is executed for every binop with its new left-hand
and right-hand side according to the precedence rules defined earlier.
In this example, we create an AST in the Pratt parser.

```rust
pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer => Expr::Integer(primary.as_str().parse::<i32>().unwrap()),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule)
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .parse(pairs)

}
```

Here's an example of how to use the parser.

```rust
fn main() -> io::Result<()> {
    for line in io::stdin().lock().lines() {
        match CalculatorParser::parse(Rule::equation, &line?) {
            Ok(mut pairs) => {
                println!(
                    "Parsed: {:#?}",
                    parse_expr(
                        // inner of expr
                        pairs.next().unwrap().into_inner()
                    )
                );
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }
    Ok(())
}

```

With this, we can parse the following simple equation:
```
> 1 * 2 + 3 / 4
Parsed: BinOp {
    lhs: BinOp {
        lhs: Integer( 1 ),
        op: Multiply,
        rhs: Integer( 2 ),
    },
    op: Add,
    rhs: BinOp {
        lhs: Integer( 3 ),
        op: Divide,
        rhs: Integer( 4 ),
    },
}
```

## Unary minus and parenthesis
So far, our calculator can parse fairly complicated expressions, but it will fail if it encounters explicit parentheses 
or a unary minus sign. Let's fix that.

### Parentheses
Consider the expression `(1 + 2) * 3`. Clearly removing the parentheses would give a different result, so we must 
support parsing such expressions. Luckily, this can be a simple addition to our `atom` rule:

```diff
- atom = _{ integer }
+ atom = _{ integer | "(" ~ expr ~ ")" }
```

Earlier we said that atoms should be simple token sequences that cannot be split up further, but now an atom can contain
arbitrary expressions! The reason we are okay with this is that the parentheses mark clear boundaries for the 
expression, it will not make ambiguous what operators belong to the inner expression and which to the outer one.

### Unary minus
We can currently only parse positive integers, eg `16` or `2342`. But we also want to do calculations with negative intergers.
To do this, we introduce the unary minus, so we can make `-4` and `-(8 + 15)`.
We need the following change to grammar:
```pest
+ unary_minus = { "-" }
+ primary = _{ integer | "(" ~ expr ~ ")" }
- atom = _{ integer | "(" ~ expr ~ ")" }
+ atom = _{ unary_minus? ~ primary }
```

For these last changes we've omitted the small changes to the AST and parsing logic (using `map_prefix`).

You can find all these details in 
the repository: [github.com/pest-parser/book/tree/master/examples/pest-calculator](https://github.com/pest-parser/book/tree/master/examples/pest-calculator).