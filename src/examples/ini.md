# Example: INI

INI (short for *initialization*) files are simple configuration files. Since
there is no standard for the format, we'll write a program that is able to
parse this example file:

```ini
username = noha
password = plain_text
salt = NaCl

[server_1]
interface=eth0
ip=127.0.0.1
document_root=/var/www/example.org

[empty_section]

[second_server]
document_root=/var/www/example.com
ip=
interface=eth1
```

Each line contains a **key and value** separated by an equals sign; or contains
a **section name** surrounded by square brackets; or else is **blank** and has
no meaning.

Whenever a section name appears, the following keys and values belong to that
section, until the next section name. The key&ndash;value pairs at the
beginning of the file belong to an implicit "empty" section.

## Writing the grammar

Start by [initializing a new project] using Cargo, adding the dependencies
`pest = "2.0"` and `pest_derive = "2.0"`. Make a new file, `src/ini.pest`, to
hold the grammar.

The text of interest in our file &mdash; `username`, `/var/www/example.org`,
*etc.* &mdash; consists of only a few characters. Let's make a rule to
recognize a single character in that set. The built-in rule
`ASCII_ALPHANUMERIC` is a shortcut to represent any uppercase or lowercase
ASCII letter, or any digit.

```pest
char = { ASCII_ALPHANUMERIC | "." | "_" | "/" }
```

Section names and property keys *must not* be empty, but property values *may*
be empty (as in the line `ip=` above). That is, the former consist of one or
more characters, `char+`; and the latter consist of zero or more characters,
`char*`. We separate the meaning into two rules:

```pest
name = { char+ }
value = { char* }
```

Now it's easy to express the two kinds of input lines.

```pest
section = { "[" ~ name ~ "]" }
property = { name ~ "=" ~ value }
```

Finally, we need a rule to represent an entire input file. The expression
`(section | property)?` matches `section`, `property`, or else nothing. Using
the built-in rule `NEWLINE` to match line endings:

```pest
file = {
    SOI ~
    ((section | property)? ~ NEWLINE)* ~
    EOI
}
```

To compile the parser into Rust, we need to add the following to `src/main.rs`:

```rust
extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "ini.pest"]
pub struct INIParser;
```

## Program initialization

Now we can read the file and parse it with `pest`:

```rust
use std::collections::HashMap;
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("config.ini").expect("cannot read file");

    let file = INIParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next().unwrap(); // get and unwrap the `file` rule; never fails

    // ...
}
```

We'll express the properties list using nested [`HashMap`]s. The outer hash map
will have section names as keys and section contents (inner hash maps) as
values. Each inner hash map will have property keys and property values. For
example, to access the `document_root` of `server_1`, we could write
`properties["server_1"]["document_root"]`. The implicit "empty" section will be
represented by a regular section with an empty string `""` for the name, so
that `properties[""]["salt"]` is valid.

```rust
fn main() {
    // ...

    let mut properties: HashMap<&str, HashMap<&str, &str>> = HashMap::new();

    // ...
}
```

Note that the hash map keys and values are all `&str`, borrowed strings. `pest`
parsers do not copy the input they parse; they borrow it. All methods for
inspecting a parse result return strings which are borrowed from the original
parsed string.

## The main loop

Now we interpret the parse result. We loop through each line of the file, which
is either a section name or a key&ndash;value property pair. If we encounter a
section name, we update a variable. If we encounter a property pair, we obtain
a reference to the hash map for the current section, then insert the pair into
that hash map.

```rust
    // ...

    let mut current_section_name = "";

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::section => {
                let mut inner_rules = line.into_inner(); // { name }
                current_section_name = inner_rules.next().unwrap().as_str();
            }
            Rule::property => {
                let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                let name: &str = inner_rules.next().unwrap().as_str();
                let value: &str = inner_rules.next().unwrap().as_str();

                // Insert an empty inner hash map if the outer hash map hasn't
                // seen this section name before.
                let section = properties.entry(current_section_name).or_default();
                section.insert(name, value);
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    // ...
```

For output, let's simply dump the hash map using [the pretty-printed `Debug`
format].

```rust
fn main() {
    // ...

    println!("{:#?}", properties);
}
```

## Whitespace

If you copy the example INI file at the top of this chapter into a file
`config.ini` and run the program, it will not parse. We have forgotten about
the optional spaces around equals signs!

Handling whitespace can be inconvenient for large grammars. Explicitly writing
a `whitespace` rule and manually inserting it makes a grammar difficult to read
and modify. `pest` provides a solution using [the special rule `WHITESPACE`].
If defined, it will be implicitly run, as many times as possible, at every
tilde `~` and between every repetition (for example, `*` and `+`). For our INI
parser, only spaces are legal whitespace.

```pest
WHITESPACE = _{ " " }
```

We mark the `WHITESPACE` rule [*silent*] with a leading low line (underscore)
`_{ ... }`. This way, even if it matches, it won't show up inside other rules.
If it weren't silent, parsing would be much more complicated, since every call
to `Pairs::next(...)` could potentially return `Rule::WHITESPACE` instead of
the desired next regular rule.

But wait! Spaces shouldn't be allowed in section names, keys, or values!
Currently, whitespace is automatically inserted between characters in `name = {
char+ }`. Rules that *are* whitespace-sensitive need to be marked [*atomic*]
with a leading at sign `@{ ... }`. In atomic rules, automatic whitespace
handling is disabled, and interior rules are silent.

```pest
name = @{ char+ }
value = @{ char* }
```

## Done

Try it out! Make sure that the file `config.ini` exists, then run the program!
You should see something like this:

```shell
$ cargo run
  [ ... ]
{
    "": {
        "password": "plain_text",
        "username": "noha",
        "salt": "NaCl"
    },
    "second_server": {
        "ip": "",
        "document_root": "/var/www/example.com",
        "interface": "eth1"
    },
    "server_1": {
        "interface": "eth0",
        "document_root": "/var/www/example.org",
        "ip": "127.0.0.1"
    }
}
```

[initializing a new project]: csv.md#setup
[`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
[the pretty-printed `Debug` format]: https://doc.rust-lang.org/std/fmt/index.html#sign0
[the special rule `WHITESPACE`]: ../grammars/syntax.md#implicit-whitespace
[*silent*]: ../grammars/syntax.md#silent-and-atomic-rules
[*atomic*]: ../grammars/syntax.md#atomic
