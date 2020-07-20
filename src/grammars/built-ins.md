# Built-in rules

Besides `ANY`, matching any single Unicode character, `pest` provides several
rules to make parsing text more convenient.

## ASCII rules

Among the printable ASCII characters, it is often useful to match alphabetic
characters and numbers. For **numbers**, `pest` provides digits in common
radixes (bases):

| Built-in rule         | Equivalent                                    |
|:---------------------:|:---------------------------------------------:|
| `ASCII_DIGIT`         | `'0'..'9'`                                    |
| `ASCII_NONZERO_DIGIT` | `'1'..'9'`                                    |
| `ASCII_BIN_DIGIT`     | `'0'..'1'`                                    |
| `ASCII_OCT_DIGIT`     | `'0'..'7'`                                    |
| `ASCII_HEX_DIGIT`     | <code>'0'..'9' \| 'a'..'f' \| 'A'..'F'</code> |

For **alphabetic** characters, distinguishing between uppercase and lowercase:

| Built-in rule       | Equivalent                        |
|:-------------------:|:---------------------------------:|
| `ASCII_ALPHA_LOWER` | `'a'..'z'`                        |
| `ASCII_ALPHA_UPPER` | `'A'..'Z'`                        |
| `ASCII_ALPHA`       | <code>'a'..'z' \| 'A'..'Z'</code> |

And for **miscellaneous** use:

| Built-in rule        | Meaning              | Equivalent                              |
|:--------------------:|:--------------------:|:---------------------------------------:|
| `ASCII_ALPHANUMERIC` | any digit or letter  | <code>ASCII_DIGIT \| ASCII_ALPHA</code> |
| `NEWLINE`            | any line feed format | <code>"\n" \| "\r\n" \| "\r"</code>     |

## Unicode rules

To make it easier to correctly parse arbitrary Unicode text, `pest` includes a
large number of rules corresponding to Unicode character properties. These
rules are divided into **general category** and **binary property** rules.

Unicode characters are partitioned into categories based on their general
purpose. Every character belongs to a single category, in the same way that
every ASCII character is a control character, a digit, a letter, a symbol, or a
space.

In addition, every Unicode character has a list of binary properties (true or
false) that it does or does not satisfy. Characters can belong to any number of
these properties, depending on their meaning.

For example, the character "A", "Latin capital letter A", is in the general
category "Uppercase Letter" because its general purpose is being a letter. It
has the binary property "Uppercase" but not "Emoji". By contrast, the character
"&#x1F170;", "negative squared Latin capital letter A", is in the general
category "Other Symbol" because it does not generally occur as a letter in
text. It has both the binary properties "Uppercase" and "Emoji".

For more details, consult Chapter 4 of [The Unicode Standard].

[The Unicode Standard]: https://www.unicode.org/versions/latest/

### General categories

Formally, categories are non-overlapping: each Unicode character belongs to
exactly one category, and no category contains another. However, since certain
groups of categories are often useful together, `pest` exposes the hierarchy of
categories below. For example, the rule `CASED_LETTER` is not technically a
Unicode general category; it instead matches characters that are
`UPPERCASE_LETTER` or `LOWERCASE_LETTER`, which *are* general categories.

- `LETTER`
  - `CASED_LETTER`
    - `UPPERCASE_LETTER`
    - `LOWERCASE_LETTER`
  - `TITLECASE_LETTER`
  - `MODIFIER_LETTER`
  - `OTHER_LETTER`
- `MARK`
  - `NONSPACING_MARK`
  - `SPACING_MARK`
  - `ENCLOSING_MARK`
- `NUMBER`
  - `DECIMAL_NUMBER`
  - `LETTER_NUMBER`
  - `OTHER_NUMBER`
- `PUNCTUATION`
  - `CONNECTOR_PUNCTUATION`
  - `DASH_PUNCTUATION`
  - `OPEN_PUNCTUATION`
  - `CLOSE_PUNCTUATION`
  - `INITIAL_PUNCTUATION`
  - `FINAL_PUNCTUATION`
  - `OTHER_PUNCTUATION`
- `SYMBOL`
  - `MATH_SYMBOL`
  - `CURRENCY_SYMBOL`
  - `MODIFIER_SYMBOL`
  - `OTHER_SYMBOL`
- `SEPARATOR`
  - `SPACE_SEPARATOR`
  - `LINE_SEPARATOR`
  - `PARAGRAPH_SEPARATOR`
- `OTHER`
  - `CONTROL`
  - `FORMAT`
  - `SURROGATE`
  - `PRIVATE_USE`
  - `UNASSIGNED`

### Binary properties

Many of these properties are used to define Unicode text algorithms, such as
[the bidirectional algorithm] and [the text segmentation algorithm]. Such
properties are not likely to be useful for most parsers.

However, the properties `XID_START` and `XID_CONTINUE` are particularly notable
because they are defined "to assist in the standard treatment of identifiers",
"such as programming language variables". See [Technical Report 31] for more
details.

[the bidirectional algorithm]: https://www.unicode.org/reports/tr9/
[the text segmentation algorithm]: https://www.unicode.org/reports/tr29/
[Technical Report 31]: https://www.unicode.org/reports/tr31/

- `ALPHABETIC`
- `BIDI_CONTROL`
- `BIDI_MIRRORED`
- `CASE_IGNORABLE`
- `CASED`
- `CHANGES_WHEN_CASEFOLDED`
- `CHANGES_WHEN_CASEMAPPED`
- `CHANGES_WHEN_LOWERCASED`
- `CHANGES_WHEN_TITLECASED`
- `CHANGES_WHEN_UPPERCASED`
- `DASH`
- `DEFAULT_IGNORABLE_CODE_POINT`
- `DEPRECATED`
- `DIACRITIC`
- `EMOJI`
- `EMOJI_COMPONENT`
- `EMOJI_MODIFIER`
- `EMOJI_MODIFIER_BASE`
- `EMOJI_PRESENTATION`
- `EXTENDED_PICTOGRAPHIC`
- `EXTENDER`
- `GRAPHEME_BASE`
- `GRAPHEME_EXTEND`
- `GRAPHEME_LINK`
- `HEX_DIGIT`
- `HYPHEN`
- `IDS_BINARY_OPERATOR`
- `IDS_TRINARY_OPERATOR`
- `ID_CONTINUE`
- `ID_START`
- `IDEOGRAPHIC`
- `JOIN_CONTROL`
- `LOGICAL_ORDER_EXCEPTION`
- `LOWERCASE`
- `MATH`
- `NONCHARACTER_CODE_POINT`
- `OTHER_ALPHABETIC`
- `OTHER_DEFAULT_IGNORABLE_CODE_POINT`
- `OTHER_GRAPHEME_EXTEND`
- `OTHER_ID_CONTINUE`
- `OTHER_ID_START`
- `OTHER_LOWERCASE`
- `OTHER_MATH`
- `OTHER_UPPERCASE`
- `PATTERN_SYNTAX`
- `PATTERN_WHITE_SPACE`
- `PREPENDED_CONCATENATION_MARK`
- `QUOTATION_MARK`
- `RADICAL`
- `REGIONAL_INDICATOR`
- `SENTENCE_TERMINAL`
- `SOFT_DOTTED`
- `TERMINAL_PUNCTUATION`
- `UNIFIED_IDEOGRAPH`
- `UPPERCASE`
- `VARIATION_SELECTOR`
- `WHITE_SPACE`
- `XID_CONTINUE`
- `XID_START`
