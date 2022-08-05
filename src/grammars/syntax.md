# Syntax of pest grammars

`pest` grammars are lists of rules. Rules are defined like this:

```pest
my_rule = { ... }

another_rule = {        // comments are preceded by two slashes
    ...                 // whitespace goes anywhere
}
```

Since rule names are translated into Rust enum variants, they are not allowed
to be Rust keywords.

The left curly bracket `{` defining a rule can be preceded by [symbols that
affect its operation]:

```pest
silent_rule = _{ ... }
atomic_rule = @{ ... }
```

[symbols that affect its operation]: #silent-and-atomic-rules

## Expressions

Grammar rules are built from *expressions* (hence "parsing expression
grammar"). These expressions are a terse, formal description of how to parse an
input string.

Expressions are composable: they can be built out of other expressions and
nested inside of each other to produce arbitrarily complex rules (although you
should break very complicated expressions into multiple rules to make them
easier to manage).

PEG expressions are suitable for both high-level meaning, like "a function
signature, followed by a function body", and low-level meaning, like "a
semicolon, followed by a line feed". The combining form "followed by",
the [sequence operator], is the same in either case.

[sequence operator]: #sequence

### Terminals

The most basic rule is a **literal string** in double quotes: `"text"`.

A string can be **case-insensitive** (for ASCII characters only) if preceded by
a caret: `^"text"`.

A single **character in a range** is written as two single-quoted characters,
separated by two dots: `'0'..'9'`.

You can match **any single character** at all with the special rule `ANY`. This
is equivalent to `'\u{00}'..'\u{10FFFF}'`, any single Unicode character.

```
"a literal string"
^"ASCII case-insensitive string"
'a'..'z'
ANY
```

Finally, you can **refer to other rules** by writing their names directly, and
even **use rules recursively**:

```pest
my_rule = { "slithy " ~ other_rule }
other_rule = { "toves" }
recursive_rule = { "mimsy " ~ recursive_rule }
```

### Sequence

The sequence operator is written as a tilde `~`.

```
first ~ and_then

("abc") ~ (^"def") ~ ('g'..'z')        // matches "abcDEFr"
```

When matching a sequence expression, `first` is attempted. If `first` matches
successfully, `and_then` is attempted next. However, if `first` fails, the
entire expression fails.

A list of expressions can be chained together with sequences, which indicates
that *all* of the components must occur, in the specified order.

### Ordered choice

The choice operator is written as a vertical line `|`.

```
first | or_else

("abc") | (^"def") | ('g'..'z')        // matches "DEF"
```

When matching a choice expression, `first` is attempted. If `first` matches
successfully, the entire expression *succeeds immediately*. However, if `first`
fails, `or_else` is attempted next.

Note that `first` and `or_else` are always attempted at the same position, even
if `first` matched some input before it failed. When encountering a parse
failure, the engine will try the next ordered choice as though no input had
been matched. Failed parses never consume any input.

```pest
start = { "Beware " ~ creature }
creature = {
    ("the " ~ "Jabberwock")
    | ("the " ~ "Jubjub bird")
}
```

```
"Beware the Jubjub bird"
 ^ (start) Parses via the second choice of `creature`,
           even though the first choice matched "the " successfully.
```

It is somewhat tempting to borrow terminology and think of this operation as
"alternation" or simply "OR", but this is misleading. The word "choice" is used
specifically because [the operation is *not* merely logical "OR"].

[the operation is *not* merely logical "OR"]: peg.html#ordered-choice

### Repetition

There are two repetition operators: the asterisk `*` and plus sign `+`. They
are placed after an expression. The asterisk `*` indicates that the preceding
expression can occur **zero or more** times. The plus sign `+` indicates that
the preceding expression can occur **one or more** times (it must occur at
least once).

The question mark operator `?` is similar, except it indicates that the
expression is **optional** &mdash; it can occur zero or one times.

```
("zero" ~ "or" ~ "more")*
 ("one" | "or" | "more")+
           (^"optional")?
```

Note that `expr*` and `expr?` will always succeed, because they are allowed to
match zero times. For example, `"a"* ~ "b"?` will succeed even on an empty
input string.

Other **numbers of repetitions** can be indicated using curly brackets:

```
expr{n}           // exactly n repetitions
expr{m, n}        // between m and n repetitions, inclusive

expr{, n}         // at most n repetitions
expr{m, }         // at least m repetitions
```

Thus `expr*` is equivalent to `expr{0, }`; `expr+` is equivalent to `expr{1,
}`; and `expr?` is equivalent to `expr{0, 1}`.

### Predicates

Preceding an expression with an ampersand `&` or exclamation mark `!` turns it
into a *predicate* that never consumes any input. You might know these
operators as "lookahead" or "non-progressing".

The **positive predicate**, written as an ampersand `&`, attempts to match its
inner expression. If the inner expression succeeds, parsing continues, but at
the *same position* as the predicate &mdash; `&foo ~ bar` is thus a kind of
"AND" statement: "the input string must match `foo` AND `bar`". If the inner
expression fails, the whole expression fails too.

The **negative predicate**, written as an exclamation mark `!`, attempts to
match its inner expression. If the inner expression *fails*, the predicate
*succeeds* and parsing continues at the same position as the predicate. If the
inner expression *succeeds*, the predicate *fails* &mdash; `!foo ~ bar` is thus
a kind of "NOT" statement: "the input string must match `bar` but NOT `foo`".

This leads to the common idiom meaning "any character but":

```pest
not_space_or_tab = {
    !(                // if the following text is not
        " "           //     a space
        | "\t"        //     or a tab
    )
    ~ ANY             // then consume one character
}

triple_quoted_string = {
    "'''"
    ~ triple_quoted_character*
    ~ "'''"
}
triple_quoted_character = {
    !"'''"        // if the following text is not three apostrophes
    ~ ANY         // then consume one character
}
```

## Operator precedence and grouping (WIP)

The repetition operators asterisk `*`, plus sign `+`, and question mark `?`
apply to the immediately preceding expression.

```
"One " ~ "or " ~ "more. "+
"One " ~ "or " ~ ("more. "+)
    are equivalent and match
"One or more. more. more. more. "
```

Larger expressions can be repeated by surrounding them with parentheses.

```
("One " ~ "or " ~ "more. ")+
    matches
"One or more. One or more. "
```

Repetition operators have the highest precedence, followed by predicate
operators, the sequence operator, and finally ordered choice.

```pest
my_rule = {
    "a"* ~ "b"?
    | &"b"+ ~ "a"
}

// equivalent to

my_rule = {
      ( ("a"*) ~ ("b"?) )
    | ( (&("b"+)) ~ "a" )
}
```

## Start and end of input

The rules `SOI` and `EOI` match the *start* and *end* of the input string,
respectively. Neither consumes any text. They only indicate whether the parser
is currently at one edge of the input.

For example, to ensure that a rule matches the entire input, where any syntax
error results in a failed parse (rather than a successful but incomplete
parse):

```pest
main = {
    SOI
    ~ (...)
    ~ EOI
}
```

## Implicit whitespace

Many languages and text formats allow arbitrary whitespace and comments between
logical tokens. For instance, Rust considers `4+5` equivalent to `4 + 5` and `4
/* comment */ + 5`.

The **optional rules `WHITESPACE` and `COMMENT`** implement this behaviour. If
either (or both) are defined, they will be implicitly inserted at every
[sequence] and between every [repetition] (except in [atomic rules]).

```pest
expression = { "4" ~ "+" ~ "5" }
WHITESPACE = _{ " " }
COMMENT = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" }
```

```
"4+5"
"4 + 5"
"4  +     5"
"4 /* comment */ + 5"
```

As you can see, `WHITESPACE` and `COMMENT` are run repeatedly, so they need
only match a single whitespace character or a single comment. The grammar above
is equivalent to:

```pest
expression = {
    "4"   ~ (ws | com)*
    ~ "+" ~ (ws | com)*
    ~ "5"
}
ws = _{ " " }
com = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" }
```

Note that implicit whitespace is *not* inserted at the beginning or end of rules
&mdash; for instance, `expression` does *not* match `" 4+5 "`. If you want to
include implicit whitespace at the beginning and end of a rule, you will need to
sandwich it between two empty rules (often `SOI` and `EOI` [as above]):

```pest
WHITESPACE = _{ " " }
expression = { "4" ~ "+" ~ "5" }
main = { SOI ~ expression ~ EOI }
```

```
"4+5"
"  4 + 5   "
```

(Be sure to mark the `WHITESPACE` and `COMMENT` rules as [silent] unless you
want to see them included inside other rules!)

[sequence]: #sequence
[repetition]: #repetition
[atomic rules]: #atomic
[as above]: #start-and-end-of-input
[silent]: #silent-and-atomic-rules

## Silent and atomic rules

**Silent** rules are just like normal rules &mdash; when run, they function the
same way &mdash; except they do not produce [pairs] or [tokens]. If a rule is
silent, it will never appear in a parse result.

To make a silent rule, precede the left curly bracket `{` with a low line
(underscore) `_`.

```pest
silent = _{ ... }
```

[pairs]: ../parser_api.html#pairs
[tokens]: ../parser_api.html#tokens

### Atomic

`pest` has two kinds of atomic rules: **atomic** and **compound atomic**. To
make one, write the sigil before the left curly bracket `{`.

```pest
atomic = @{ ... }
compound_atomic = ${ ... }
```

Both kinds of atomic rule prevent [implicit whitespace]: inside an atomic rule,
the tilde `~` means "immediately followed by", and [repetition operators]
(asterisk `*` and plus sign `+`) have no implicit separation. In addition, all
other rules called from an atomic rule are also treated as atomic.

The difference between the two is how they produce tokens for inner rules. In
an atomic rule, interior matching rules are [silent]. By contrast, compound
atomic rules produce inner tokens as normal.

Atomic rules are useful when the text you are parsing ignores whitespace except
in a few cases, such as literal strings. In this instance, you can write
`WHITESPACE` or `COMMENT` rules, then make your string-matching rule be atomic.

[implicit whitespace]: #implicit-whitespace
[repetition operators]: #repetition
[silent]: #silent-and-atomic-rules

### Non-atomic

Sometimes, you'll want to cancel the effects of atomic parsing. For instance,
you might want to have string interpolation with an expression inside, where
the inside expression can still have whitespace like normal.

```python
#!/bin/env python3
print(f"The answer is {2 + 4}.")
```

This is where you use a **non-atomic** rule. Write an exclamation mark `!` in
front of the defining curly bracket. The rule will run as non-atomic, whether
it is called from an atomic rule or not.

```pest
fstring = @{ "\"" ~ ... }
expr = !{ ... }
```

## The stack (WIP)

`pest` maintains a stack that can be manipulated directly from the grammar. An
expression can be matched and pushed onto the stack with the keyword `PUSH`,
then later matched exactly with the keywords `PEEK` and `POP`.

Using the stack allows *the exact same text* to be matched multiple times,
rather than *the same pattern*.

For example,

```pest
same_text = {
    PUSH( "a" | "b" | "c" )
    ~ POP
}
same_pattern = {
    ("a" | "b" | "c")
    ~ ("a" | "b" | "c")
}
```

In this case, `same_pattern` will match `"ab"`, while `same_text` will not.

One practical use is in parsing Rust ["raw string literals"], which look like
this:

```rust
const raw_str: &str = r###"
    Some number of number signs # followed by a quotation mark ".

    Quotation marks can be used anywhere inside: """"""""",
    as long as one is not followed by a matching number of number signs,
    which ends the string: "###;
```

When parsing a raw string, we have to keep track of how many number signs `#`
occurred before the quotation mark. We can do this using the stack:

```pest
raw_string = {
    "r" ~ PUSH("#"*) ~ "\""    // push the number signs onto the stack
    ~ raw_string_interior
    ~ "\"" ~ POP               // match a quotation mark and the number signs
}
raw_string_interior = {
    (
        !("\"" ~ PEEK)    // unless the next character is a quotation mark
                          // followed by the correct amount of number signs,
        ~ ANY             // consume one character
    )*
}
```

["raw string literals"]: https://doc.rust-lang.org/book/second-edition/appendix-02-operators.html#non-operator-symbols

# Cheat sheet

| Syntax           | Meaning                           | Syntax                  | Meaning              |
|:----------------:|:---------------------------------:|:-----------------------:|:--------------------:|
| `foo =  { ... }` | [regular rule]                    | `baz = @{ ... }`        | [atomic]             |
| `bar = _{ ... }` | [silent]                          | `qux = ${ ... }`        | [compound-atomic]    |
|                  |                                   | `plugh = !{ ... }`      | [non-atomic]         |
| `"abc"`          | [exact string]                    | `^"abc"`                | [case insensitive]   |
| `'a'..'z'`       | [character range]                 | `ANY`                   | [any character]      |
| `foo ~ bar`      | [sequence]                        | <code>baz \| qux</code> | [ordered choice]     |
| `foo*`           | [zero or more]                    | `bar+`                  | [one or more]        |
| `baz?`           | [optional]                        | `qux{n}`                | [exactly *n*]        |
| `qux{m, n}`      | [between *m* and *n* (inclusive)] |                         |                      |
| `&foo`           | [positive predicate]              | `!bar`                  | [negative predicate] |
| `PUSH(baz)`      | [match and push]                  |                         |                      |
| `POP`            | [match and pop]                   | `PEEK`                  | [match without pop]  |

[regular rule]: #syntax-of-pest-parsers
[silent]: #silent-and-atomic-rules
[atomic]: #atomic
[compound-atomic]: #atomic
[non-atomic]: #non-atomic
[exact string]: #terminals
[case insensitive]: #terminals
[character range]: #terminals
[any character]: #terminals
[sequence]: #sequence
[ordered choice]: #ordered-choice
[zero or more]: #repetition
[one or more]: #repetition
[optional]: #repetition
[exactly *n*]: #repetition
[between *m* and *n* (inclusive)]: #repetition
[positive predicate]: #predicates
[negative predicate]: #predicates
[match and push]: #the-stack-wip
[match and pop]: #the-stack-wip
[match without pop]: #the-stack-wip
