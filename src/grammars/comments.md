## Comments

### Non-doc comments

Comments follow the general Rust style of line (`//`) and block (`/* ... */`) comment forms.
Non-doc comments are interpreted as a form of whitespace.

```pest
/* 
  Block comment
 */
another_rule = {        // line comment
    ...                 // whitespace goes anywhere
}
```

### Doc comments

Line doc comments beginning with exactly three slashes `///`.
And `//!` for document the entire of grammar file.

```pest
//! A parser for JSON file.

json = { ... }

/// Matches object, e.g.: `{ "foo": "bar" }`
object = { ... }
```

Then will get

```rust
/// A parser for JSON file.
enum Rule {
    json,
    /// Matches object, e.g.: `{ "foo": "bar" }`
    object,
}
```
