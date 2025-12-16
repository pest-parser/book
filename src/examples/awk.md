# Final project: AWK clone

This chapter walks through the creation of a complete AWK clone, providing a comprehensive example of how to build a domain-specific language using `pest`. This project demonstrates many advanced parsing concepts and showcases a complete Rust ecosystem solution for language implementation.

[AWK](http://pubs.opengroup.org/onlinepubs/9699919799/utilities/awk.html) is a pattern-scanning and data-extraction language that excels at processing structured text files. Our implementation will support:
- Pattern-action programming model
- Regular expression matching
- Field and record processing
- Built-in variables and functions
- Arithmetic and string operations
- Control flow constructs

## What We'll Build

By the end of this tutorial, you'll have a fully functional AWK interpreter capable of running programs like:

```awk
BEGIN { print "Processing employee data..." }
/Engineer/ { engineers++ }
$2 > 30 { print $1, "is over 30 years old" }
END { print "Found", engineers, "engineers" }
```

This project showcases several common Rust ecosystem crates and concepts.

## Dependencies and Ecosystem

Before diving into the implementation, let's understand the key dependencies our AWK clone will use:

### Core Dependencies

```toml
[dependencies]
pest = "2.8"
pest_derive = "2.8"
clap = { version = "4", features = ["derive"] }
regex = "1.5"
anyhow = "1"
```

Besides the dependencies for `pest`, we make use of the following crates:

**`clap`**: A powerful command-line argument parser that provides automatic help generation, type safety, and excellent error messages. We'll use it to handle AWK's various command-line options.

**`regex`**: Rust's regex engine, which we'll use both for AWK's regex pattern matching and internally for field splitting.

**`anyhow`**: A trait object-based error handling library that provides excellent error context which is perfect for a language interpreter where various errors can occur at parsing, compilation, or runtime.

## Understanding AWK's Structure

AWK programs follow a simple but powerful pattern-action model:
```awk
pattern { action }
```

Patterns determine when actions execute:
- `BEGIN` - executes before processing any input
- `END` - executes after processing all input
- `/regex/` - executes when the regex matches the current record
- `expression` - executes when the expression evaluates to true
- No pattern - executes for every record

Actions contain statements that manipulate data, control flow, and produce output.

## Grammar Design

Our grammar needs to handle AWK's dual nature as both a pattern matching language and a general-purpose programming language. Let's build it step by step.

### Lexical Elements

First, we define the basic tokens (numbers, strings, identifiers, and operators):

```pest
WHITESPACE = _{ " " | "\t" | "\n" | "\r" }
COMMENT = _{ "#" ~ (!"\n" ~ ANY)* }

// Numbers support both integers and floating-point
number = @{ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT*)? }

// Strings are double-quoted (AWK also supports single quotes, but we'll keep it simple)
string = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }

// Identifiers start with letter or underscore, contain alphanumeric or underscore
identifier = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
```

The `@` modifier makes these rules *atomic*, meaning implicit comments or whitespaces are not processed within them.

### AWK-Specific Constructs

AWK has several unique syntactic elements:

```pest
// Field references: $0 (whole record), $1 (first field), $2 (second field), etc.
field_ref = { "$" ~ atom_expr }

// Atomic expressions for field references to ensure proper operator precedence
atom_expr = _{ number | identifier | builtin_var | "(" ~ expr ~ ")" }

// Built-in variables that AWK provides
builtin_var = { "NR" | "NF" | "FS" | "RS" | "OFS" | "ORS" }

// Regular expression literals enclosed in forward slashes
regex = @{ "/" ~ (!"/" ~ ANY)* ~ "/" }
```

The critical insight here is the `atom_expr` rule. Field references must bind only to simple expressions to ensure that `$2 > 25` parses as `($2) > 25`, not `$(2 > 25)`. This demonstrates the importance of operator precedence in grammar design.

### Operator Definitions

AWK supports a rich set of operators with specific precedence rules:

```pest
// Arithmetic operators
add = { "+" }
subtract = { "-" }
multiply = { "*" }
divide = { "/" }
modulo = { "%" }
power = { "^" | "**" }

// Assignment operators
assign = { "=" }
add_assign = { "+=" }
sub_assign = { "-=" }
mul_assign = { "*=" }
div_assign = { "/=" }
mod_assign = { "%=" }

// Comparison operators
eq = { "==" }
ne = { "!=" }
le = { "<=" }  // Must come before "<" to avoid tokenizing "<=" as "<" + "=" in the `infix_op` rule later
ge = { ">=" }  // Must come before ">" to avoid tokenizing ">=" as ">" + "=" in the `infix_op` rule later
lt = { "<" }
gt = { ">" }
not_match = { "!~" }
match_op = { "~" }

// Logical operators
logical_and = { "&&" }
logical_or = { "||" }
logical_not = { "!" }

// Increment/decrement
increment = { "++" }
decrement = { "--" }
```

Notice how longer operators are defined before shorter ones in the `infix_op` rule later: this prevents tokenization conflicts where `<=` might be incorrectly parsed as `<` followed by `=`.

### Expression Grammar

Our expression grammar uses a Pratt parser for proper operator precedence:

```pest
// Primary expressions (atoms that cannot be broken down further)
atom = _{
    number |
    string |
    field_ref |
    builtin_var |
    identifier |
    "(" ~ expr ~ ")"
}

// Expression structure for Pratt parser
expr = { atom ~ (infix_op ~ atom)* }

// All infix operations that the Pratt parser will handle
infix_op = _{
    add | subtract | multiply | divide | modulo | power |
    eq | ne | le | ge | lt | gt | not_match | match_op |
    logical_and | logical_or
}
```

This grammar provides the input format that the Pratt parser expects: atoms separated by operators.

### Statements and Control Flow

AWK supports various statement types:

```pest
// Variable and field assignment
assignment = { (identifier | field_ref) ~ (assign | add_assign | sub_assign | mul_assign | div_assign | mod_assign) ~ expr }

// Increment and decrement statements
increment_stmt = { (identifier | field_ref) ~ increment }
decrement_stmt = { (identifier | field_ref) ~ decrement }

// Print statement with optional arguments
print_stmt = { "print" ~ print_args? }
print_args = { expr ~ ("," ~ expr)* }

// Control flow
if_stmt = { "if" ~ "(" ~ expr ~ ")" ~ statement ~ ("else" ~ statement)? }
while_stmt = { "while" ~ "(" ~ expr ~ ")" ~ statement }
for_stmt = { "for" ~ "(" ~ assignment? ~ ";" ~ expr? ~ ";" ~ assignment? ~ ")" ~ statement }

// Block statements
block = { "{" ~ statement* ~ "}" }

// Expression as statement
expr_stmt = { expr }

// Union of all statement types
statement = _{
    assignment |
    increment_stmt |
    decrement_stmt |
    print_stmt |
    if_stmt |
    while_stmt |
    for_stmt |
    block |
    expr_stmt
}
```

### Top-Level Program Structure

Finally, we define AWK's top-level structure:

```pest
// Pattern types
begin_pattern = { "BEGIN" }
end_pattern = { "END" }
regex_pattern = { regex }
expr_pattern = { expr }

pattern = _{
    begin_pattern |
    end_pattern |
    regex_pattern |
    expr_pattern
}

// AWK rules: pattern + action, action only, or pattern only
rule = { (pattern ~ action) | action | pattern }
action = { "{" ~ statement* ~ "}" }

// Complete program
program = { SOI ~ rule* ~ EOI }
```

## Abstract Syntax Tree Design

The Abstract Syntax Tree (AST) is the in-memory representation of our parsed AWK program. It bridges the gap between the textual source code and the executable representation. Our AST design follows common patterns used in language implementation.

### Value System

AWK is dynamically typed with two main value types:

```rust
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
}

impl Value {
    // Convert any value to a number (AWK semantics)
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::String(s) => s.parse().unwrap_or(0.0),
        }
    }

    // AWK truthiness: 0 and empty string are false, everything else is true
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
        }
    }
}

// Implement Display trait for idiomatic string conversion
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64) // Print integers without decimal
                } else {
                    write!(f, "{n}")
                }
            }
            Value::String(s) => write!(f, "{s}"),
        }
    }
}
```

**Why use `Display` instead of a custom `to_string` method?** The `Display` trait is Rust's standard way to define how types should be formatted as strings. By implementing `Display`, we get several benefits:

1. **Automatic `to_string()` method**: Any type that implements `Display` automatically gets a `to_string()` method
2. **Integration with formatting macros**: Our values work seamlessly with `println!`, `format!`, and other formatting macros
3. **Idiomatic Rust**: Following standard library conventions makes our code more familiar to Rust developers
4. **Consistency**: The same formatting logic is used whether we call `to_string()` or use the value in string interpolation

This approach also demonstrates proper Rust trait usage: rather than defining our own string conversion method, we implement the standard trait that the ecosystem expects.

### Expression AST

Expressions form the core of AWK's computational model:

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    // Literal values
    Number(f64),
    String(String),

    // Variable references
    Identifier(String),           // User-defined variables
    BuiltinVar(String),          // Built-in variables like NR, NF

    // Field access: $1, $2, $NF, etc.
    FieldRef(Box<Expr>),         // The Box prevents infinite recursion

    // Binary operations: arithmetic, comparison, logical
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    // Unary operations: negation, logical not
    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
    },
}
```

The use of `Box<Expr>` is essential in Rust because it allows recursive types. Without boxing, the compiler cannot determine the size of the enum at compile time.

### Operators

We model AWK's rich operator set with enums:

```rust
#[derive(Debug, Clone)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Pow,

    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,

    // Pattern matching
    Match,          // ~  (regex match)
    NotMatch,       // !~ (regex non-match)

    // Logical
    And, Or,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,    // Arithmetic negation: -x
    Not,    // Logical negation: !x
}
```

### Statement AST

Statements represent executable actions in AWK:

```rust
#[derive(Debug, Clone)]
pub enum Statement {
    // Variable assignment: x = 5, $1 = "hello", y += 3
    Assignment {
        target: AssignTarget,
        op: AssignOp,
        value: Expr,
    },

    // Increment/decrement: x++, --y, $1++
    Increment(AssignTarget),
    Decrement(AssignTarget),

    // Print statement: print, print $1, print x, y, z
    Print(Vec<Expr>),

    // Control flow
    If {
        condition: Expr,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },

    While {
        condition: Expr,
        body: Box<Statement>,
    },

    For {
        init: Option<Box<Statement>>,
        condition: Option<Expr>,
        update: Option<Box<Statement>>,
        body: Box<Statement>,
    },

    // Block statement: { stmt1; stmt2; stmt3 }
    Block(Vec<Statement>),

    // Expression as statement (for side effects)
    Expression(Expr),
}

// Assignment targets: variables or field references
#[derive(Debug, Clone)]
pub enum AssignTarget {
    Identifier(String),    // Regular variable: x = 5
    FieldRef(Expr),       // Field reference: $1 = "hello"
}

// Assignment operators
#[derive(Debug, Clone)]
pub enum AssignOp {
    Assign,              // =
    AddAssign,           // +=
    SubAssign,           // -=
    MulAssign,           // *=
    DivAssign,           // /=
    ModAssign,           // %=
}
```

### Program Structure

The top-level AST represents the entire AWK program:

```rust
#[derive(Debug, Clone)]
pub enum Pattern {
    Begin,                    // BEGIN pattern
    End,                      // END pattern
    Regex(String),           // /regex/ pattern
    Expression(Expr),        // Expression pattern: $2 > 30
}

#[derive(Debug, Clone)]
pub struct AwkRule {
    pub pattern: Option<Pattern>,    // None means the rule always executes
    pub action: Vec<Statement>,      // Statements to execute when pattern matches
}

#[derive(Debug, Clone)]
pub struct Program {
    pub rules: Vec<AwkRule>,           // All rules in the program
}
```

This design captures AWK's rule-based structure while providing a clean foundation for our interpreter.

## Parser Implementation

The parser transforms pest's parse tree into our AST. This involves traversing the tree structure that pest generates and constructing appropriate AST nodes. Our implementation uses the Pratt parser for expression parsing to handle operator precedence correctly.

### Parser Setup and Dependencies

```rust
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
use anyhow::{Result, anyhow};
use crate::ast::*;

#[derive(pest_derive::Parser)]
#[grammar = "awk.pest"]
pub struct AwkParser;
```

The `#[derive(pest_derive::Parser)]` attribute generates the parser code from our grammar file. The `#[grammar = "awk.pest"]` attribute tells the macro where to find our grammar.

### Pratt Parser Configuration

The Pratt parser handles operator precedence and associativity automatically. We need to configure it with AWK's operator precedence rules:

```rust
use std::sync::LazyLock;

static PRATT_PARSER: LazyLock<PrattParser<Rule>) = LazyLock::new(|| {
    use Rule::*;
    use Assoc::*;

    PrattParser::new()
        // Lowest precedence first
        .op(Op::infix(logical_or, Left))           // ||
        .op(Op::infix(logical_and, Left))          // &&
        .op(Op::infix(eq, Left) | Op::infix(ne, Left))                    // == !=
        .op(Op::infix(le, Left) | Op::infix(ge, Left) | Op::infix(lt, Left) | Op::infix(gt, Left))  // <= >= < >
        .op(Op::infix(match_op, Left) | Op::infix(not_match, Left))       // ~ !~
        .op(Op::infix(add, Left) | Op::infix(subtract, Left))             // + -
        .op(Op::infix(multiply, Left) | Op::infix(divide, Left) | Op::infix(modulo, Left))  // * / %
        .op(Op::infix(power, Right))               // ^ ** (right-associative)
        // Highest precedence
        .op(Op::prefix(logical_not) | Op::prefix(subtract))               // ! - (unary)
});
```

**Why `LazyLock`?** The Pratt parser configuration is expensive to compute and should be shared across all parsing operations. `LazyLock` allows us to compute it once at runtime and reuse it.

**Precedence Levels**: AWK follows standard mathematical precedence:
1. Unary operators (`!`, `-`) - highest precedence
2. Power (`^`, `**`) - right-associative
3. Multiplicative (`*`, `/`, `%`)
4. Additive (`+`, `-`)
5. Pattern matching (`~`, `!~`)
6. Relational (`<`, `<=`, `>`, `>=`)
7. Equality (`==`, `!=`)
8. Logical AND (`&&`)
9. Logical OR (`||`) - lowest precedence

### Top-Level Parsing

```rust
pub fn parse_program(input: &str) -> Result<Program> {
    let mut pairs = AwkParser::parse(Rule::program, input)?;
    let program_pair = pairs.next().unwrap();

    let mut rules = Vec::new();
    for rule_pair in program_pair.into_inner() {
        if rule_pair.as_rule() == Rule::rule {
            rules.push(parse_rule(rule_pair)?);
        }
    }

    Ok(Program { rules })
}
```

The `AwkParser::parse()` method returns a `Result<Pairs<Rule>, pest::error::Error>`. If parsing succeeds, we get an iterator over `Pair<Rule>` representing the parse tree.

### Rule Parsing

AWK rules can have various forms:
- `pattern { action }` - Execute action when pattern matches
- `{ action }` - Execute action for every record (no pattern)
- `pattern` - Print the record if pattern matches (no explicit action)

```rust
fn parse_rule(pair: Pair<Rule>) -> Result<AwkRule> {
    let mut pattern = None;
    let mut action = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::begin_pattern => pattern = Some(Pattern::Begin),
            Rule::end_pattern => pattern = Some(Pattern::End),
            Rule::regex_pattern => {
                let regex_str = inner.into_inner().next().unwrap().as_str();
                // Remove the surrounding slashes from /regex/
                let regex_content = &regex_str[1..regex_str.len()-1];
                pattern = Some(Pattern::Regex(regex_content.to_string()));
            },
            Rule::expr_pattern => {
                pattern = Some(Pattern::Expression(parse_expr(inner.into_inner())?));
            },
            Rule::action => {
                for stmt_pair in inner.into_inner() {
                    action.push(parse_statement(stmt_pair)?);
                }
            },
            _ => {},
        }
    }

    Ok(AwkRule { pattern, action })
}
```

### Expression Parsing with Pratt Parser

The Pratt parser uses three callback functions to build the AST:

```rust
fn parse_expr(pairs: Pairs<Rule>) -> Result<Expr> {
    PRATT_PARSER
        .map_primary(|primary| {
            // Handle primary expressions (atoms)
            match primary.as_rule() {
                Rule::number => {
                    Ok(Expr::Number(primary.as_str().parse().unwrap()))
                },
                Rule::string => {
                    let s = primary.as_str();
                    // Remove surrounding quotes
                    Ok(Expr::String(s[1..s.len()-1].to_string()))
                },
                Rule::identifier => {
                    Ok(Expr::Identifier(primary.as_str().to_string()))
                },
                Rule::builtin_var => {
                    Ok(Expr::BuiltinVar(primary.as_str().to_string()))
                },
                Rule::field_ref => {
                    let expr = parse_atom_expr(primary.into_inner())?;
                    Ok(Expr::FieldRef(Box::new(expr)))
                },
                Rule::expr => {
                    // Parenthesized expression
                    parse_expr(primary.into_inner())
                },
                _ => Err(anyhow!("Unexpected primary: {:?}", primary)),
            }
        })
        .map_infix(|lhs, op, rhs| {
            // Handle binary operations
            let bin_op = match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::subtract => BinOp::Sub,
                Rule::multiply => BinOp::Mul,
                Rule::divide => BinOp::Div,
                Rule::modulo => BinOp::Mod,
                Rule::power => BinOp::Pow,
                Rule::eq => BinOp::Eq,
                Rule::ne => BinOp::Ne,
                Rule::le => BinOp::Le,
                Rule::ge => BinOp::Ge,
                Rule::lt => BinOp::Lt,
                Rule::gt => BinOp::Gt,
                Rule::match_op => BinOp::Match,
                Rule::not_match => BinOp::NotMatch,
                Rule::logical_and => BinOp::And,
                Rule::logical_or => BinOp::Or,
                _ => return Err(anyhow!("Unexpected infix op: {:?}", op)),
            };
            Ok(Expr::BinaryOp {
                op: bin_op,
                left: Box::new(lhs?),
                right: Box::new(rhs?),
            })
        })
        .map_prefix(|op, rhs| {
            // Handle unary operations
            let un_op = match op.as_rule() {
                Rule::subtract => UnOp::Neg,
                Rule::logical_not => UnOp::Not,
                _ => return Err(anyhow!("Unexpected prefix op: {:?}", op)),
            };
            Ok(Expr::UnaryOp {
                op: un_op,
                operand: Box::new(rhs?),
            })
        })
        .parse(pairs)
}
```

### Field Reference Parsing

Field references require special handling to ensure proper precedence:

```rust
fn parse_atom_expr(pairs: Pairs<Rule>) -> Result<Expr> {
    let mut pairs = pairs;
    let primary = pairs.next().unwrap();

    match primary.as_rule() {
        Rule::number => {
            Ok(Expr::Number(primary.as_str().parse().unwrap()))
        },
        Rule::identifier => {
            Ok(Expr::Identifier(primary.as_str().to_string()))
        },
        Rule::builtin_var => {
            Ok(Expr::BuiltinVar(primary.as_str().to_string()))
        },
        Rule::expr => {
            // Parenthesized expression: $(expr)
            parse_expr(primary.into_inner())
        },
        _ => Err(anyhow!("Unexpected atom expression: {:?}", primary)),
    }
}
```

### Statement Parsing

Statement parsing involves pattern matching on the rule type and recursively parsing sub-expressions:

```rust
fn parse_statement(pair: Pair<Rule>) -> Result<Statement> {
    match pair.as_rule() {
        Rule::print_stmt => {
            let mut exprs = Vec::new();
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::print_args {
                    for arg_pair in inner.into_inner() {
                        if arg_pair.as_rule() == Rule::expr {
                            exprs.push(parse_expr(arg_pair.into_inner())?);
                        }
                    }
                }
            }
            Ok(Statement::Print(exprs))
        },
        Rule::assignment => {
            let mut inner = pair.into_inner();
            let target_pair = inner.next().unwrap();
            let op_pair = inner.next().unwrap();
            let value_pair = inner.next().unwrap();

            let target = match target_pair.as_rule() {
                Rule::identifier => AssignTarget::Identifier(target_pair.as_str().to_string()),
                Rule::field_ref => {
                    let expr = parse_atom_expr(target_pair.into_inner())?;
                    AssignTarget::FieldRef(expr)
                },
                _ => return Err(anyhow!("Invalid assignment target")),
            };

            let op = match op_pair.as_rule() {
                Rule::assign => AssignOp::Assign,
                Rule::add_assign => AssignOp::AddAssign,
                Rule::sub_assign => AssignOp::SubAssign,
                Rule::mul_assign => AssignOp::MulAssign,
                Rule::div_assign => AssignOp::DivAssign,
                Rule::mod_assign => AssignOp::ModAssign,
                _ => return Err(anyhow!("Invalid assignment operator")),
            };

            let value = parse_expr(value_pair.into_inner())?;

            Ok(Statement::Assignment { target, op, value })
        },
        Rule::increment_stmt => {
            let mut inner = pair.into_inner();
            let target_pair = inner.next().unwrap();

            let target = match target_pair.as_rule() {
                Rule::identifier => AssignTarget::Identifier(target_pair.as_str().to_string()),
                Rule::field_ref => {
                    let expr = parse_atom_expr(target_pair.into_inner())?;
                    AssignTarget::FieldRef(expr)
                },
                _ => return Err(anyhow!("Invalid increment target")),
            };

            Ok(Statement::Increment(target))
        },
        Rule::decrement_stmt => {
            let mut inner = pair.into_inner();
            let target_pair = inner.next().unwrap();

            let target = match target_pair.as_rule() {
                Rule::identifier => AssignTarget::Identifier(target_pair.as_str().to_string()),
                Rule::field_ref => {
                    let expr = parse_atom_expr(target_pair.into_inner())?;
                    AssignTarget::FieldRef(expr)
                },
                _ => return Err(anyhow!("Invalid decrement target")),
            };

            Ok(Statement::Decrement(target))
        },
        Rule::if_stmt => {
            let mut inner = pair.into_inner();
            let condition = parse_expr(inner.next().unwrap().into_inner())?;
            let then_stmt = Box::new(parse_statement(inner.next().unwrap())?);
            let else_stmt = if let Some(else_pair) = inner.next() {
                Some(Box::new(parse_statement(else_pair)?))
            } else {
                None
            };
            Ok(Statement::If { condition, then_stmt, else_stmt })
        },
        Rule::while_stmt => {
            let mut inner = pair.into_inner();
            let condition_pair = inner.next().unwrap();
            let condition = parse_expr(condition_pair.into_inner())?;
            let body = Box::new(parse_statement(inner.next().unwrap())?);
            Ok(Statement::While { condition, body })
        },
        Rule::for_stmt => {
            let inner = pair.into_inner();
            let mut init = None;
            let mut condition = None;
            let mut update = None;
            let mut body = None;

            for part in inner {
                match part.as_rule() {
                    Rule::assignment => {
                        if init.is_none() {
                            init = Some(Box::new(parse_statement(part)?));
                        } else {
                            update = Some(Box::new(parse_statement(part)?));
                        }
                    },
                    Rule::expr => {
                        condition = Some(parse_expr(part.into_inner())?);
                    },
                    Rule::statement => {
                        body = Some(Box::new(parse_statement(part)?));
                    },
                    _ => {},
                }
            }

            Ok(Statement::For {
                init,
                condition,
                update,
                body: body.unwrap(),
            })
        },
        Rule::block => {
            let mut statements = Vec::new();
            for inner in pair.into_inner() {
                statements.push(parse_statement(inner)?);
            }
            Ok(Statement::Block(statements))
        },
        Rule::expr_stmt => {
            Ok(Statement::Expression(parse_expr(pair.into_inner())?))
        },
        _ => Err(anyhow!("Unsupported statement type: {:?}", pair.as_rule())),
    }
}
```

## Interpreter Implementation

The interpreter is the execution engine that brings our parsed AWK program to life. It maintains the runtime state, processes input data, and executes the parsed statements. Our interpreter follows the traditional pattern of separating parsing from execution, which provides clean error handling and debugging capabilities.

### Interpreter Structure and State Management

The interpreter maintains several critical pieces of state that reflect AWK's execution model:

```rust
use std::collections::HashMap;
use regex::Regex;
use anyhow::{Result, anyhow};
use crate::ast::*;

pub struct Interpreter {
    // Variable storage for user-defined and built-in variables
    variables: HashMap<String, Value>,

    // Current record split into fields: fields[0] = whole record, fields[1] = $1, etc.
    fields: Vec<String>,

    // Built-in variable state
    record_number: usize,           // NR - Number of records processed
    field_separator: String,        // FS - Field separator
    output_field_separator: String, // OFS - Output field separator
    record_separator: String,       // RS - Record separator
    output_record_separator: String, // ORS - Output record separator
}
```

**Why `HashMap` for variables?** AWK variables are dynamically created and accessed by string names. A hash map provides O(1) average-case lookup and insertion, which is essential for performance when processing large datasets.

**Why `Vec<String>` for fields?** AWK's field model is array-like with `$0`, `$1`, `$2`, etc. A vector provides efficient indexed access while allowing dynamic growth when fields are assigned beyond the current record length.

### Interpreter Initialization

```rust
impl Interpreter {
    pub fn new() -> Self {
        let mut variables = HashMap::new();

        // Initialize AWK's built-in variables with default values
        variables.insert("FS".to_string(), Value::String(" ".to_string()));
        variables.insert("OFS".to_string(), Value::String(" ".to_string()));
        variables.insert("RS".to_string(), Value::String("\n".to_string()));
        variables.insert("ORS".to_string(), Value::String("\n".to_string()));

        Self {
            variables,
            fields: Vec::new(),
            record_number: 0,
            field_separator: " ".to_string(),
            output_field_separator: " ".to_string(),
            record_separator: "\n".to_string(),
            output_record_separator: "\n".to_string(),
        }
    }

    // Allow external configuration of field separator (for -F command line option)
    pub fn set_field_separator(&mut self, fs: String) {
        self.field_separator = fs.clone();
        self.variables.insert("FS".to_string(), Value::String(fs));
    }
}
```

### The Main Execution Loop

The heart of the interpreter is the `run_program` method, which implements AWK's three-phase execution model:

```rust
pub fn run_program(&mut self, program: &Program, input: &str) -> Result<()> {
    // Phase 1: Execute all BEGIN rules before processing any input
    for rule in &program.rules {
        if let Some(Pattern::Begin) = rule.pattern {
            self.execute_statements(&rule.action)?;
        }
    }

    // Phase 2: Process input record by record
    let records: Vec<&str> = if self.record_separator == "\n" {
        input.lines().collect()  // Default: split on newlines
    } else {
        input.split(&self.record_separator).collect()  // Custom record separator
    };

    for record in records {
        if record.is_empty() {
            continue;  // Skip empty records
        }

        // Update record state
        self.record_number += 1;
        self.split_fields(record);

        // Execute pattern-matching rules for this record
        for rule in &program.rules {
            match &rule.pattern {
                Some(Pattern::Regex(regex_str)) => {
                    let regex = Regex::new(regex_str)?;
                    if regex.is_match(record) {
                        self.execute_statements(&rule.action)?;
                    }
                },
                Some(Pattern::Expression(expr)) => {
                    if self.eval_expr(expr)?.is_truthy() {
                        if rule.action.is_empty() {
                            // If action is empty, print $0
                            println!("{}", self.fields[0]);
                        } else {
                            self.execute_statements(&rule.action)?;
                        }
                    }
                },
                None => {
                    // No pattern means always execute
                    self.execute_statements(&rule.action)?;
                },
                _ => {}, // Skip BEGIN/END here
            }
        }
    }

    // Phase 3: Execute all END rules after processing all input
    for rule in &program.rules {
        if let Some(Pattern::End) = rule.pattern {
            self.execute_statements(&rule.action)?;
        }
    }

    Ok(())
}
```

**Why this three-phase approach?** AWK's semantics require that all BEGIN rules execute before any input processing, and all END rules execute after all input is processed. This design makes the interpreter's behavior predictable and matches AWK's specification.

### Field Splitting and Record Processing

Field splitting is a core AWK operation that converts each input record into numbered fields:

```rust
fn split_fields(&mut self, line: &str) {
    self.fields = vec![line.to_string()]; // $0 is always the entire record

    if self.field_separator == " " {
        // AWK's default behavior: split on any whitespace, ignore leading/trailing
        self.fields.extend(
            line.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
    } else {
        // Custom field separator: split exactly on that string
        self.fields.extend(
            line.split(&self.field_separator)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        );
    }

    // Update built-in variables
    self.variables.insert("NF".to_string(),
        Value::Number((self.fields.len() - 1) as f64));
    self.variables.insert("NR".to_string(),
        Value::Number(self.record_number as f64));
}
```

**Default whitespace splitting**: When FS is a single space (the default), AWK treats any sequence of whitespace as a field separator and ignores leading/trailing whitespace. This is different from splitting on a literal space character.

### Expression Evaluation

Expression evaluation is where the computational model comes to life. The interpreter recursively evaluates the AST:

```rust
fn eval_expr(&self, expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Number(n) => Ok(Value::Number(*n)),
        Expr::String(s) => Ok(Value::String(s.clone())),

        Expr::Identifier(name) => {
            // Variable lookup with AWK semantics: undefined variables are empty strings
            Ok(self.variables.get(name)
                .cloned()
                .unwrap_or(Value::String("".to_string())))
        },

        Expr::BuiltinVar(name) => {
            // Built-in variables default to 0 if not found
            Ok(self.variables.get(name)
                .cloned()
                .unwrap_or(Value::Number(0.0)))
        },

        Expr::FieldRef(index_expr) => {
            let index = self.eval_expr(index_expr)?;
            let idx = index.to_number() as usize;

            // AWK allows accessing fields beyond NF (returns empty string)
            if idx < self.fields.len() {
                Ok(Value::String(self.fields[idx].clone()))
            } else {
                Ok(Value::String("".to_string()))
            }
        },

        Expr::BinaryOp { op, left, right } => {
            let lval = self.eval_expr(left)?;
            let rval = self.eval_expr(right)?;
            self.eval_binary_op(op, &lval, &rval)
        },

        Expr::UnaryOp { op, operand } => {
            let val = self.eval_expr(operand)?;
            self.eval_unary_op(op, &val)
        },
    }
}
```

### Binary Operation Implementation

AWK's binary operations require careful handling of type coercion and AWK-specific semantics:

```rust
fn eval_binary_op(&self, op: &BinOp, left: &Value, right: &Value) -> Result<Value> {
    match op {
        // Arithmetic operations: always work on numeric values
        BinOp::Add => Ok(Value::Number(left.to_number() + right.to_number())),
        BinOp::Sub => Ok(Value::Number(left.to_number() - right.to_number())),
        BinOp::Mul => Ok(Value::Number(left.to_number() * right.to_number())),
        BinOp::Div => {
            let divisor = right.to_number();
            if divisor == 0.0 {
                return Err(anyhow!("Division by zero"));
            }
            Ok(Value::Number(left.to_number() / divisor))
        }
        BinOp::Mod => {
            let divisor = right.to_number();
            if divisor == 0.0 {
                return Err(anyhow!("Modulo by zero"));
            }
            Ok(Value::Number(left.to_number() % divisor))
        }
        BinOp::Pow => Ok(Value::Number(left.to_number().powf(right.to_number()))),

        // Comparison operations: return 1.0 for true, 0.0 for false
        BinOp::Eq => Ok(Value::Number(if self.values_equal(left, right) {
            1.0
        } else {
            0.0
        })),
        BinOp::Ne => Ok(Value::Number(if !self.values_equal(left, right) {
            1.0
        } else {
            0.0
        })),
        BinOp::Lt => Ok(Value::Number(if left.to_number() < right.to_number() {
            1.0
        } else {
            0.0
        })),
        BinOp::Le => Ok(Value::Number(if left.to_number() <= right.to_number() {
            1.0
        } else {
            0.0
        })),
        BinOp::Gt => Ok(Value::Number(if left.to_number() > right.to_number() {
            1.0
        } else {
            0.0
        })),
        BinOp::Ge => Ok(Value::Number(if left.to_number() >= right.to_number() {
            1.0
        } else {
            0.0
        })),

        // Pattern matching operations: use regex crate
        BinOp::Match => {
            let text = left.to_string(); // Display trait converts Value to String
            let pattern = right.to_string(); // Works for both Number and String variants
            let regex = Regex::new(&pattern)?;
            Ok(Value::Number(if regex.is_match(&text) { 1.0 } else { 0.0 }))
        }
        BinOp::NotMatch => {
            let text = left.to_string(); // Display trait ensures consistent formatting
            let pattern = right.to_string();
            let regex = Regex::new(&pattern)?;
            Ok(Value::Number(if !regex.is_match(&text) {
                1.0
            } else {
                0.0
            }))
        }

        // Logical operations: use AWK truthiness rules
        BinOp::And => Ok(Value::Number(if left.is_truthy() && right.is_truthy() {
            1.0
        } else {
            0.0
        })),
        BinOp::Or => Ok(Value::Number(if left.is_truthy() || right.is_truthy() {
            1.0
        } else {
            0.0
        })),
    }
}

fn eval_unary_op(&self, op: &UnOp, operand: &Value) -> Result<Value> {
    match op {
        UnOp::Neg => Ok(Value::Number(-operand.to_number())),
        UnOp::Not => Ok(Value::Number(if operand.is_truthy() { 0.0 } else { 1.0 })),
    }
}

fn values_equal(&self, left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        _ => {
            left.to_string() == right.to_string()
        }
    }
}
```

**Why `anyhow::Result`?** Regular expression compilation can fail, and `anyhow` provides excellent error context and propagation, making debugging easier when processing fails.

### Statement Execution

Statement execution implements AWK's imperative programming features:

```rust
fn execute_statements(&mut self, statements: &[Statement]) -> Result<()> {
    for stmt in statements {
        self.execute_statement(stmt)?;
    }
    Ok(())
}

fn execute_statement(&mut self, stmt: &Statement) -> Result<()> {
    match stmt {
        Statement::Print(exprs) => {
            if exprs.is_empty() {
                // Print $0
                print!("{}{}", self.fields[0], self.output_record_separator);
            } else {
                let values: Result<Vec<_>> = exprs.iter()
                    .map(|expr| self.eval_expr(expr))
                    .collect();
                let values = values?;
                let output: Vec<String> = values.iter()
                    .map(|v| v.to_string())
                    .collect();
                print!("{}{}", output.join(&self.output_field_separator), self.output_record_separator);
            }
        },
        Statement::Assignment { target, op, value } => {
            let new_value = self.eval_expr(value)?;
            match target {
                AssignTarget::Identifier(name) => {
                    let final_value = match op {
                        AssignOp::Assign => new_value,
                        AssignOp::AddAssign => {
                            let current = self.variables.get(name)
                                .cloned()
                                .unwrap_or(Value::Number(0.0));
                            Value::Number(current.to_number() + new_value.to_number())
                        },
                        AssignOp::SubAssign => {
                            let current = self.variables.get(name)
                                .cloned()
                                .unwrap_or(Value::Number(0.0));
                            Value::Number(current.to_number() - new_value.to_number())
                        },
                        AssignOp::MulAssign => {
                            let current = self.variables.get(name)
                                .cloned()
                                .unwrap_or(Value::Number(0.0));
                            Value::Number(current.to_number() * new_value.to_number())
                        },
                        AssignOp::DivAssign => {
                            let current = self.variables.get(name)
                                .cloned()
                                .unwrap_or(Value::Number(0.0));
                            Value::Number(current.to_number() / new_value.to_number())
                        },
                        AssignOp::ModAssign => {
                            let current = self.variables.get(name)
                                .cloned()
                                .unwrap_or(Value::Number(0.0));
                            Value::Number(current.to_number() % new_value.to_number())
                        },
                    };
                    self.variables.insert(name.clone(), final_value);

                    // Update separators if built-in variables are modified
                    if name == "FS" {
                        self.field_separator = self.variables.get("FS").unwrap().to_string();
                    } else if name == "OFS" {
                        self.output_field_separator = self.variables.get("OFS").unwrap().to_string();
                    } else if name == "RS" {
                        self.record_separator = self.variables.get("RS").unwrap().to_string();
                    } else if name == "ORS" {
                        self.output_record_separator = self.variables.get("ORS").unwrap().to_string();
                    }
                },
                AssignTarget::FieldRef(field_expr) => {
                    let index = self.eval_expr(field_expr)?;
                    let idx = index.to_number() as usize;

                    // Extend fields array if necessary
                    while self.fields.len() <= idx {
                        self.fields.push("".to_string());
                    }

                    self.fields[idx] = new_value.to_string();

                    // Update NF
                    self.variables.insert("NF".to_string(),
                        Value::Number((self.fields.len() - 1) as f64));
                },
            }
        },
        Statement::Increment(target) => {
            match target {
                AssignTarget::Identifier(name) => {
                    let current = self.variables.get(name)
                        .cloned()
                        .unwrap_or(Value::Number(0.0));
                    let new_value = Value::Number(current.to_number() + 1.0);
                    self.variables.insert(name.clone(), new_value);
                },
                AssignTarget::FieldRef(field_expr) => {
                    let index = self.eval_expr(field_expr)?;
                    let idx = index.to_number() as usize;

                    while self.fields.len() <= idx {
                        self.fields.push("".to_string());
                    }

                    let current = self.fields[idx].parse::<f64>().unwrap_or(0.0);
                    self.fields[idx] = (current + 1.0).to_string();

                    self.variables.insert("NF".to_string(),
                        Value::Number((self.fields.len() - 1) as f64));
                },
            }
        },
        Statement::Decrement(target) => {
            match target {
                AssignTarget::Identifier(name) => {
                    let current = self.variables.get(name)
                        .cloned()
                        .unwrap_or(Value::Number(0.0));
                    let new_value = Value::Number(current.to_number() - 1.0);
                    self.variables.insert(name.clone(), new_value);
                },
                AssignTarget::FieldRef(field_expr) => {
                    let index = self.eval_expr(field_expr)?;
                    let idx = index.to_number() as usize;

                    while self.fields.len() <= idx {
                        self.fields.push("".to_string());
                    }

                    let current = self.fields[idx].parse::<f64>().unwrap_or(0.0);
                    self.fields[idx] = (current - 1.0).to_string();

                    self.variables.insert("NF".to_string(),
                        Value::Number((self.fields.len() - 1) as f64));
                },
            }
        },
        Statement::If { condition, then_stmt, else_stmt } => {
            let cond_val = self.eval_expr(condition)?;
            if cond_val.is_truthy() {
                self.execute_statement(then_stmt)?;
            } else if let Some(else_branch) = else_stmt {
                self.execute_statement(else_branch)?;
            }
        },
        Statement::While { condition, body } => {
            while self.eval_expr(condition)?.is_truthy() {
                self.execute_statement(body)?;
            }
        },
        Statement::For { init, condition, update, body } => {
            if let Some(init_stmt) = init {
                self.execute_statement(init_stmt)?;
            }

            loop {
                if let Some(cond) = condition {
                    if !self.eval_expr(cond)?.is_truthy() {
                        break;
                    }
                }

                self.execute_statement(body)?;

                if let Some(update_stmt) = update {
                    self.execute_statement(update_stmt)?;
                }
            }
        },
        Statement::Block(statements) => {
            self.execute_statements(statements)?;
        },
        Statement::Expression(expr) => {
            self.eval_expr(expr)?;
        },
    }
    Ok(())
}
```

This interpreter implementation demonstrates several key principles:
- **Separation of concerns**: Parsing and execution are cleanly separated
- **Error propagation**: Using `anyhow::Result` throughout for robust error handling
- **AWK semantics**: Faithful implementation of AWK's type coercion and truthiness rules
- **State management**: Proper synchronization between built-in variables and internal state

## Command-Line Interface with Clap

The final component is a command-line interface that makes our AWK clone usable as a standalone tool. We use `clap` for argument parsing, which provides excellent ergonomics and automatic help generation.

### Project Structure and Dependencies

First, let's set up our `Cargo.toml` with all the dependencies we've discussed:

```toml
[package]
name = "awk-clone"
version = "0.1.0"
edition = "2021"

[dependencies]
pest = "2.8"
pest_derive = "2.8"
clap = { version = "4", features = ["derive"] }
regex = "1.5"
anyhow = "1"
```

### Main Application Structure

Our main application demonstrates how to integrate all the components we've built:

```rust
use clap::{Arg, Command};
use std::fs;
use std::io::{self, Read};
use anyhow::Result;

// Import our modules
mod ast;
mod parser;
mod interpreter;

use parser::*;
use interpreter::*;

fn main() -> Result<()> {
    let matches = Command::new("awk-clone")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("A simple AWK clone built with pest")
        .long_about("
This AWK clone demonstrates how to build a complete language interpreter using Rust and pest.
It supports AWK's core features including pattern-action programming, field processing,
built-in variables, and control flow constructs.

Examples:
  awk-clone -p '{ print $1, $2 }' data.txt
  awk-clone -f script.awk input.txt
  cat data.txt | awk-clone -p 'BEGIN { sum = 0 } { sum += $2 } END { print sum }'
        ")
        .arg(Arg::new("program")
            .short('p')
            .long("program")
            .value_name("PROGRAM")
            .help("AWK program string")
            .long_help("AWK program as a command-line string. Cannot be used with --file.")
            .required_unless_present("file")
            .conflicts_with("file"))
        .arg(Arg::new("file")
            .short('f')
            .long("file")
            .value_name("FILE")
            .help("AWK program file")
            .long_help("Read AWK program from a file. Cannot be used with --program.")
            .required_unless_present("program")
            .conflicts_with("program"))
        .arg(Arg::new("input")
            .help("Input file (reads from stdin if not provided)")
            .long_help("Input data file to process. If not provided, reads from standard input.")
            .index(1))
        .arg(Arg::new("field-separator")
            .short('F')
            .long("field-separator")
            .value_name("FS")
            .help("Field separator pattern")
            .long_help("Set the field separator. Default is whitespace. Examples: -F ',' for CSV, -F ':' for /etc/passwd"))
        .get_matches();

    // Parse program source
    let program_text = if let Some(prog) = matches.get_one::<String>("program") {
        prog.clone()
    } else if let Some(file) = matches.get_one::<String>("file") {
        fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("Failed to read program file '{}': {}", file, e))?
    } else {
        unreachable!("clap should ensure either program or file is provided");
    };

    // Read input data
    let input_text = if let Some(input_file) = matches.get_one::<String>("input") {
        fs::read_to_string(input_file)
            .map_err(|e| anyhow::anyhow!("Failed to read input file '{}': {}", input_file, e))?
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to read from stdin: {}", e))?;
        buffer
    };

    // Parse the AWK program
    let program = parse_program(&program_text)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    // Create and configure interpreter
    let mut interpreter = Interpreter::new();

    if let Some(fs) = matches.get_one::<String>("field-separator") {
        interpreter.set_field_separator(fs.clone());
    }

    // Execute the program
    interpreter.run_program(&program, &input_text)
        .map_err(|e| anyhow::anyhow!("Runtime error: {}", e))?;

    Ok(())
}
```

### Error Handling Strategy

Notice how we use `anyhow` throughout for error handling:

1. **File I/O errors**: Clear context about which file failed to read
2. **Parse errors**: Wrapped with additional context
3. **Runtime errors**: Distinguished from parse errors
4. **Error propagation**: Using `?` operator for clean error bubbling

This demonstrates production-quality error handling where users get helpful error messages rather than cryptic debug output.

### Command-Line Interface Design

Our CLI follows Unix conventions:
- **Short and long options**: `-p`/`--program`, `-f`/`--file`, `-F`/`--field-separator`
- **Mutually exclusive options**: Can't specify both `-p` and `-f`
- **Positional arguments**: Input file is positional (optional)
- **Help and documentation**: Comprehensive help text with examples
- **Standard input/output**: Reads from stdin when no input file specified

### Integration with other crates in the Rust Ecosystem

This project demonstrates how pest can integrate with the broader Rust ecosystem:

**`clap`** provides the command-line interface:
```bash
# Automatic help generation
cargo run -- --help

# Version information
cargo run -- --version

# Error messages for invalid usage
cargo run -- -p 'program' -f 'file'  # Error: conflicting arguments
```

**`anyhow`** provides error handling:
```rust
// Context-aware error messages
Err(anyhow!("Failed to read program file '{}': {}", file, e))

// Error chaining and propagation
parse_program(&program_text)?
```

**`regex`** handles pattern matching:
```rust
// Compile-time error checking for regex patterns
let regex = Regex::new(regex_str)?;
```

### Testing the Complete Implementation

With all components integrated, you can test the AWK clone with real AWK programs:

```bash
# Basic field processing
cargo run -- -p '{ print $1, $2 }' employees.txt

# Pattern matching with regex
cargo run -- -p '/Engineer/ { print $1, "is an engineer" }' employees.txt

# Statistical processing
cargo run -- -p 'BEGIN {
                 sum = 0
                 count = 0
                }
                $2 > 25 {
                  sum += $2
                  count++
                }
                END { print "Average age over 25:", sum/count }' employees.txt

# CSV processing with custom separator
echo "name,age,dept
Alice,25,Engineering
Bob,30,Sales" | cargo run -- -F "," -p '{ print $1, ": ", $3 }'

# Reading program from file
echo '$2 > 30 { print $1, "is over 30" }' > filter.awk
cargo run -- -f filter.awk employees.txt
```

## Key Implementation Insights and Lessons Learned

Building a complete AWK clone demonstrates several critical aspects of language implementation that extend far beyond basic parsing.

### Architecture and Design Patterns

**Separation of Concerns**: Our implementation cleanly separates parsing, AST representation, and execution. This modular design makes the codebase maintainable and allows each component to be tested independently.

**Error Handling Strategy**: Using `anyhow::Result` throughout provides excellent error propagation and context. In a language interpreter, errors can occur at multiple levels:
- **Lexical/Parse errors**: Invalid syntax in the source code
- **Runtime errors**: Invalid regex patterns
- **I/O errors**: File not found, permission denied
- **System errors**: Out of memory, interrupted system calls

**State Management**: The interpreter carefully manages multiple types of state:
- **Variable bindings**: User-defined and built-in variables
- **Field state**: Current record split into indexed fields
- **Execution context**: Current record number, separators, etc.

### Advanced Language Features

**Type System**: AWK's dynamic typing with automatic coercion required careful implementation:
```rust
// AWK-style type coercion: numbers convert to strings, strings parse to numbers
impl Value {
    fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::String(s) => s.parse().unwrap_or(0.0),  // Invalid strings become 0
        }
    }
}
```

**Operator Precedence**: The Pratt parser elegantly handles complex precedence rules without the grammar becoming unwieldy. This is especially important for expression-heavy languages like AWK.

**Pattern Matching**: AWK's pattern-action model required a flexible dispatching system that can handle regex patterns, expression patterns, and special BEGIN/END patterns.

### Performance Considerations

**Regex Compilation**: Each regex pattern can be compiled once and reused, rather than recompiling on every record. This provides significant performance benefits for large datasets.

**Field Splitting**: The field splitting algorithm handles AWK's subtle whitespace splitting rules efficiently while maintaining compatibility with the original AWK semantics.

**Memory Management**: Using `Box<T>` for recursive AST nodes and `HashMap` for variable storage provides good performance characteristics while remaining memory-safe.

### Integration with the Rust Ecosystem

This project showcases how multiple Rust crates work together to solve complex problems:

**`pest`**: Provides the parsing foundation with excellent error reporting and grammar expressiveness.

**`clap`**: Delivers a professional command-line interface with minimal boilerplate code.

**`regex`**: Offers a high-performance, secure regex engine that handles AWK's pattern matching needs.

**`anyhow`**: Simplifies error handling while providing rich context for debugging.

### Real-World Applications

The techniques demonstrated in this AWK clone apply to many other language implementation scenarios:

**Domain-Specific Languages (DSLs)**: The pattern of grammar → AST → interpreter is fundamental to DSL implementation.

**Configuration Languages**: Many configuration systems benefit from AWK-like pattern matching and field processing.

**Data Processing Tools**: The field-oriented processing model is excellent for log analysis, CSV processing, and data transformation tasks.

**Scripting Languages**: The combination of pattern matching and imperative programming makes AWK-style languages very practical for automation.

### Testing and Quality Assurance

A production language implementation requires comprehensive testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_splitting() {
        let mut interpreter = Interpreter::new();
        interpreter.split_fields("Alice 25 Engineer");
        assert_eq!(interpreter.fields[0], "Alice 25 Engineer");  // $0
        assert_eq!(interpreter.fields[1], "Alice");              // $1
        assert_eq!(interpreter.fields[2], "25");                 // $2
        assert_eq!(interpreter.fields[3], "Engineer");           // $3
    }

    #[test]
    fn test_expression_evaluation() {
        let interpreter = Interpreter::new();
        // Test that "25" > "100" evaluates correctly (numerically)
        let expr = Expr::BinaryOp {
            op: BinOp::Gt,
            left: Box::new(Expr::String("25".to_string())),
            right: Box::new(Expr::String("100".to_string())),
        };
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result.to_number(), 0.0);  // 25 > 100 is false
    }
}
```

### Performance Benchmarking

For a language interpreter, performance testing with realistic workloads is crucial:

```bash
# Generate test data
seq 1 100000 | awk '{print "user" $1, int(rand()*100), "dept" int(rand()*10)}' > large_dataset.txt

# Benchmark against GNU AWK
time awk '$2 > 50 {
            sum += $2
            count++
        }
        END { print sum/count }' large_dataset.txt
time ./target/release/awk-clone -p '$2 > 50 {
            sum += $2
            count++
        }
        END { print sum/count }' large_dataset.txt
```

The real-world robust benchmarking can also use the excellent [criterion](https://crates.io/crates/criterion) crate for statistical tests and charting.

### Extensions and Future Work

The foundation we've built can be extended in many directions:

- **Additional Built-ins**: Functions like `substr()`, `length()`, `gsub()`, mathematical functions
- **Arrays**: Associative arrays are a key AWK feature we haven't implemented
- **User-defined Functions**: Function definitions and calls
- **Advanced I/O**: File I/O operations, pipes, and input redirection
- **Optimization**: Constant folding, dead code elimination, JIT compilation
- **Debugging**: Source-level debugging support with breakpoints and variable inspection

### Conclusion

Building a complete AWK clone demonstrates that modern language implementation is accessible and enjoyable with the right tools. The Rust ecosystem provides excellent building blocks, and pest makes grammar-driven parsing both powerful and maintainable.

The key insights from this project:

1. **Grammar design matters**: Careful attention to operator precedence and tokenization prevents subtle bugs
2. **Error handling is crucial**: Good error messages make the difference between a usable tool and a frustrating experience
3. **Modular architecture scales**: Separating concerns allows each component to evolve independently
4. **Idiomatic Rust patterns improve maintainability**: Using standard traits like `Display` instead of custom methods makes code more familiar and integrates better with the ecosystem
5. **Testing drives quality**: Comprehensive tests catch edge cases that informal testing misses
6. **Performance requires attention**: Language interpreters need careful optimization for real-world use

This AWK implementation serves as both a practical tool and an educational example of how to build robust, efficient language processors in Rust. The techniques and patterns demonstrated here apply broadly to any language implementation project, from simple DSLs to full-featured programming languages.

You can find the complete source code for this project in the [book's GitHub repository](https://github.com/pest-parser/book/tree/master/examples/awk).
