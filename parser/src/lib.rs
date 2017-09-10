#[macro_use]
extern crate pest;
#[macro_use]
extern crate pest_derive;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("rust.pest");

#[derive(Parser)]
#[grammar = "rust.pest"]
struct RustParser;

#[test]
fn true_lit() {
    parses_to! {
        parser: RustParser,
        input: "true",
        rule: Rule::bool,
        tokens: [
            bool(0, 4, [
                true_lit(0, 4)
            ])
        ]
    };
}

#[test]
fn false_lit() {
    parses_to! {
        parser: RustParser,
        input: "false",
        rule: Rule::bool,
        tokens: [
            bool(0, 5, [
                false_lit(0, 5)
            ])
        ]
    };
}
