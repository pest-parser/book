# Introduction

*Speed or simplicity? Why not __both__?*

The motivation behind *pest* was to come up with a parser that would maximize
usability, both when designing the grammar, and when maintaining it after
prolonged use and development. And, as Rust tradition mandates, this parser had
to make sure that its use of high-level abstractions would not hinder
performance. As such, *pest* is perfectly suited for complete beginners who want
to learn and experiment with grammars and parsing, but who do not want to be
constrained in terms of speed.

*pest* is implemented in Rust and requires some knowledge of the language in
order to be used. The reason for picking it over other languages is that it
offers C-like low level access and encourages the use of multi-threaded
programming, both of great use when writing a compiler or VM to accompany your
parser. At the same time, Rust's procedural macros and functional APIs provide a
means to express the problem of parsing in a simple and natural way. It even
comes with an [official book](https://doc.rust-lang.org/book/) to get you
started.

The final chapter of this book will explore the implementation of a subset of
Rust's own grammar, from the simplest terminals to the construction of an AST.
After reading it, you should be comfortable writing parsers with *pest*.
