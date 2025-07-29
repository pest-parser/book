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

#[derive(Debug, Clone)]
pub enum Expr {
    // Literal values
    Number(f64),
    String(String),

    // Variable references
    Identifier(String), // User-defined variables
    BuiltinVar(String), // Built-in variables like NR, NF

    // Field access: $1, $2, $NF, etc.
    FieldRef(Box<Expr>), // The Box prevents infinite recursion

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

#[derive(Debug, Clone)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Pattern matching
    Match,    // ~  (regex match)
    NotMatch, // !~ (regex non-match)

    // Logical
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg, // Arithmetic negation: -x
    Not, // Logical negation: !x
}

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
    Identifier(String), // Regular variable: x = 5
    FieldRef(Expr),     // Field reference: $1 = "hello"
}

// Assignment operators
#[derive(Debug, Clone)]
pub enum AssignOp {
    Assign,    // =
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Begin,            // BEGIN pattern
    End,              // END pattern
    Regex(String),    // /regex/ pattern
    Expression(Expr), // Expression pattern: $2 > 30
}

#[derive(Debug, Clone)]
pub struct AwkRule {
    pub pattern: Option<Pattern>, // None means the rule always executes
    pub action: Vec<Statement>,   // Statements to execute when pattern matches
}

#[derive(Debug, Clone)]
pub struct Program {
    pub rules: Vec<AwkRule>, // All rules in the program
}
