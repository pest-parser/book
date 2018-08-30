# Parsing expression grammar

Parsing expression grammars (PEGs) are simply a strict representation of the
simple imperative code that you would write if you were writing a parser by
hand.

```
digit = {           // To recognize a digit...
    '0'..'9'        //   take any character from '0' to '9'.
}
expression = {      // To recognize an expression...
    digit+          //   first take as many digits as possible (at least one)...
    | "true"        //   or, if that fails, the string "true".
}
```

In fact, `pest` produces code that is quite similar to the pseudo-code in the
comments above.

## Eagerness

When a [repetition] PEG expression is run on an input string,

```
('0'..'9')+      // one or more characters from '0' to '9'
```

it runs that expression as many times as it can (matching "eagerly", or
"greedily"). It either succeeds, consuming whatever it matched and passing the
remaining input on to the next step in the parser,

```
"42 boxes"
 ^ Running ('0'..'9')+

"42 boxes"
   ^ Successfully took one or more digits!

" boxes"
 ^ Remaining unparsed input.
```

or fails, consuming nothing.

```
"galumphing"
 ^ Running ('0'..'9')+
   Failed to take one or more digits!

"galumphing"
 ^ Remaining unparsed input (everything).
```

If an expression fails to match, the failure propagates upwards, eventually
leading to a failed parse, unless the failure is "caught" somewhere in the
grammar. The *choice operator* is one way to "catch" such failures.

[repetition]: syntax.html#repetition

## Ordered choice

The [choice operator], written as a vertical line `|`, is *ordered*. The PEG
expression `first | second` means "try `first`; but if it fails, try `second`
instead".

In many cases, the ordering does not matter. For instance, `"true" | "false"`
will match either the string `"true"` or the string `"false"` (and fail if
neither occurs).

However, sometimes the ordering *does* matter. Consider the PEG expression `"a"
| "ab"`. You might expect it to match either the string `"a"` or the string
`"ab"`. But it will not &mdash; the expression means "try `"a"`; but if it
fails, try `"ab"` instead". If you are matching on the string `"abc"`, "try
`"a"`" will *not* fail; it will instead match `"a"` successfully, leaving
`"bc"` unparsed!

In general, when writing a parser with choices, put the longest or most
specific choice first, and the shortest or most general choice last.

[choice operator]: syntax.html#ordered-choice

## Non-backtracking

During parsing, a PEG expression either succeeds or fails. If it succeeds, the
next step is performed as usual. But if it fails, the whole expression fails.
The engine will not back up and try again.

Consider this grammar, matching on the string `"frumious"`:

```
word = {     // to recognize an word...
    any*     //   take any character, zero or more times...
    ~ any    //   followed by any character
}
```

You might expect this rule to parse any input string that contains at least one
character (equivalent to `any+`). But it will not. Instead, the first `any*`
will eagerly eat the entire string &mdash; it will *succeed*. Then, the next
`any` will have nothing left, so it will fail.

```
"frumious"
 ^ (word)

"frumious"
         ^ (any*) Success! Continue to `any` with remaining input "".

""
 ^ (any) Failure! Expected one character, but found end of string.
```

In a system with backtracking (like regular expressions), you would back up one
step, "un-eating" a character, and then try again. But PEGs do not do this. In
the rule `first ~ second`, once `first` parses successfully, it has consumed
some characters that will never come back. `second` can only run on the input
that `first` did not consume.

## Unambiguous

These rules form an elegant and simple system. Every PEG rule is run on the
remainder of the input string, consuming as much input as necessary. Once a
rule is done, the rest of the input is passed on to the rest of the parser.

For instance, the expression `('0'..'9')+`, "one or more digits", will always
match the largest sequence of consecutive digits possible. There is no danger
of accidentally having a later rule back up and steal some digits in an
unintuitive and nonlocal way.

This contrasts with other parsing tools, such as regular expressions and CFGs,
where the results of a rule often depend on code some distance away. Indeed,
the famous "shift/reduce conflict" in LR parsers is not a problem in PEGs.

# Don't panic

This all might be a bit counterintuitive at first. But as you can see, the
basic logic is very easy and straightforward. You can trivially step through
the execution of any PEG expression.

- Try this.
- If it succeeds, try the next thing.
- Otherwise, try the other thing.

```
(this ~ next_thing) | (other_thing)
```

These rules together make PEGs very pleasant tools for writing a parser.
