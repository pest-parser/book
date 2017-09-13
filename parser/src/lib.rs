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

#[test]
fn zero() {
    parses_to! {
        parser: RustParser,
        input: "0",
        rule: Rule::int,
        tokens: [
            int(0, 1)
        ]
    };
}

#[test]
fn starts_with_zero() {
    parses_to! {
        parser: RustParser,
        input: "01",
        rule: Rule::int,
        tokens: [
            int(0, 2)
        ]
    };
}

#[test]
fn zero_multiple_underscores() {
    parses_to! {
        parser: RustParser,
        input: "0___",
        rule: Rule::int,
        tokens: [
            int(0, 4)
        ]
    };
}

#[test]
fn million() {
    parses_to! {
        parser: RustParser,
        input: "1_000_000",
        rule: Rule::int,
        tokens: [
            int(0, 9)
        ]
    };
}

#[test]
fn zero_point() {
    parses_to! {
        parser: RustParser,
        input: "0.",
        rule: Rule::float,
        tokens: [
            float(0, 2, [
                int(0, 1)
            ])
        ]
    };
}

#[test]
fn one_exp() {
    parses_to! {
        parser: RustParser,
        input: "1e10",
        rule: Rule::float,
        tokens: [
            float(0, 4, [
                int(0, 1),
                exp(1, 4, [
                    int(2, 4)
                ])
            ])
        ]
    };
}

#[test]
fn zero_point_exp() {
    parses_to! {
        parser: RustParser,
        input: "0.e0",
        rule: Rule::float,
        tokens: [
            float(0, 4, [
                int(0, 1),
                exp(2, 4, [
                    int(3, 4)
                ])
            ])
        ]
    };
}

#[test]
fn zero_point_zero_exp_plus() {
    parses_to! {
        parser: RustParser,
        input: "0.0e+0",
        rule: Rule::float,
        tokens: [
            float(0, 6, [
                int(0, 1),
                int(2, 3),
                exp(3, 6, [
                    plus(4, 5),
                    int(5, 6)
                ])
            ])
        ]
    };
}

#[test]
fn zero_point_zero() {
    parses_to! {
        parser: RustParser,
        input: "0.0",
        rule: Rule::float,
        tokens: [
            float(0, 3, [
                int(0, 1),
                int(2, 3)
            ])
        ]
    };
}

#[test]
fn float_with_underscores_exp_minus() {
    parses_to! {
        parser: RustParser,
        input: "0__.0__e-0__",
        rule: Rule::float,
        tokens: [
            float(0, 12, [
                int(0, 3),
                int(4, 7),
                exp(7, 12, [
                    minus(8, 9),
                    int(9, 12)
                ])
            ])
        ]
    };
}

#[test]
fn string_with_all_escape_types() {
    parses_to! {
        parser: RustParser,
        input: r#""a\nb\x0Fc\u{a}d\u{AbAbAb}e""#,
        rule: Rule::string,
        tokens: [
            string(0, 28, [
                raw_string(1, 2),
                escape(2, 4, [
                    predefined(3, 4)
                ]),
                raw_string(4, 5),
                escape(5, 9, [
                    byte(6, 9)
                ]),
                raw_string(9, 10),
                escape(10, 15, [
                    unicode(11, 15, [
                        unicode_hex(13, 14)
                    ])
                ]),
                raw_string(15, 16),
                escape(16, 26, [
                    unicode(17, 26, [
                        unicode_hex(19, 25)
                    ])
                ]),
                raw_string(26, 27)
            ])
        ]
    };
}

#[test]
fn char_without_escape() {
    parses_to! {
        parser: RustParser,
        input: "'a'",
        rule: Rule::chr,
        tokens: [
            chr(0, 3)
        ]
    };
}

#[test]
fn char_with_escape() {
    parses_to! {
        parser: RustParser,
        input: "'\\''",
        rule: Rule::chr,
        tokens: [
            chr(0, 4, [
                escape(1, 3, [
                    predefined(2, 3)
                ])
            ])
        ]
    };
}
