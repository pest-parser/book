# Syntax

Now that we have literals defined, the next step is to compose them into the
syntax of the language. This syntax will only focus on expressions, statements,
and functions as a subset of Rust. These in turn will not be complete
definitions.

## Expressions

We will define expressions as a combination of unary and infix operations, and
method calls. The operators that we will use for this subset are:

```
op_unary_minus =  { "-" }
op_unary_not   =  { "!" }
op_unary       = _{
    op_unary_minus |
    op_unary_not
}

op_plus          =  { "+" }
op_minus         =  { "-" }
op_times         =  { "*" }
op_divide        =  { "/" }
op_and           =  { "&&" }
op_or            =  { "||" }
op_greater       =  { ">" }
op_greater_equal =  { ">=" }
op_lower         =  { "<" }
op_lower_equal   =  { "<=" }
op_equal         =  { "==" }
op_infix         = _{
    op_plus |
    op_minus |
    op_times |
    op_divide |
    op_and |
    op_or |
    op_greater |
    op_greater_equal |
    op_lower |
    op_lower_equal |
    op_equal
}

paren_open  = { "(" }
paren_close = { ")" }
```

We also defined parentheses rules since they will come in handy in a bit.
Because PEGs do not support left-recursion, we will have to make sure to have
a layer of indirection when defining infix expressions, while unaries and method
calls will be defined with the use of repetitions.

The easiest way to start would be to define expressions with the highest
priorities. These expressions will be the only ones that unaries can be formed
with and methods can be called on. They are the literals defined in the previous
chapter plus expressions nested in parentheses:

```
value = {
    float | // float comes before int since they overlap
    int |
    chr |
    string |
    ident |
    paren_open ~ expr ~ paren_close
}
```

With that out of the way, a next step would be to define what a call should look
like:

```
dot   =  { "." }
comma =  { "," }
args  = _{ expr ~ (comma ~ expr)* }
call  =  { ident ~ paren_open ~ args? ~ paren_close }
```

Now we can include unaries and method calls in one single term rule that will
be used in infix expressions:

```
term = { op_unary* ~ value ~ (dot ~ call)* }
expr = { term ~ (op_infix ~ term)* }
```

Extensive testing would be handy here, especially more complex cases that
combine expression types, but also separate tests for individual behavior.

## Statements
