// Syntax highlighting for pest PEGs.

// mdBook exposes a minified version of highlight.js, so the language
// definition objects below have abbreviated property names:
//     "b"  => begin
//     "c"  => contains
//     "cN" => className
//     "e"  => end

hljs.registerLanguage("pest", function(hljs) {

    // Basic syntax.
    var comment = {cN: "comment", b: "//", e: /$/};
    var ident = {cN: "title", b: /[_a-zA-Z][_a-z0-9A-Z]*/};
    var special = {b: /COMMENT|WHITESPACE/, cN: "keyword"};

    // Escape sequences within a string or character literal.
    var escape = {b: /\\./};

    // Per highlight.js style, only built-in rules should be highlighted inside
    // a definition.
    var rule = {
        b: /[@_$!]?\{/, e: "}",
        k: {built_in: "ANY SOI EOI PUSH POP PEEK " +
                      "ASCII_ALPHANUMERIC ASCII_DIGIT ASCII_HEX_DIGIT " +
                      "ASCII_NONZERO_DIGIT NEWLINE"},
        c: [comment,
            {cN: "string", b: '"', e: '"', c: [escape]},
            {cN: "string", b: "'", e: "'", c: [escape]}]
    };

    return {
        c: [special, rule, ident, comment]
    };

});

// This file is inserted after the default highlight.js invocation, which tags
// unknown-language blocks with CSS classes but doesn't highlight them.
Array.from(document.querySelectorAll("code.language-pest"))
    .forEach(hljs.highlightBlock);
