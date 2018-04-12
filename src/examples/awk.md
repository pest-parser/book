# Final project: Awk clone (WIP)

This chapter will walk through the creation of a simple variant of [Awk] (only
loosely following the POSIX specification). It will probably have several
sections. It will provide an example of a full project based on `pest` with a
manageable grammar, a straightforward AST, and a fairly simple interpreter.

This Awk clone will support regex patterns, string and numeric variables, most
of the POSIX operators, and some functions. It will not support user-defined
functions in the interest of avoiding variable scoping.

[Awk]: http://pubs.opengroup.org/onlinepubs/9699919799/utilities/awk.html
