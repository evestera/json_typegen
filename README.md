# Swagger tools for Rust

NOTE: This code is under development and is not yet in a functional state.

This is a collection of tools for consuming APIs using [Swagger](http://swagger.io) (and in the future [OpenAPI](https://www.openapis.org)) specifications.

**swagger_cli** is a command line tool for generating code either for one-time use or e.g. in a build script.

**swagger_derive** is a library to generate code at compile time, by the way of a custom derive. It is probably more suited as a normal procedural macro, so this is something of a stopgap solution until normal procedural macros are part of stable Rust.
