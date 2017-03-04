# JSON code generation tools for Rust

NOTE: This code is under development and is not yet in a functional state.

This is a collection of tools for generating structs from JSON samples.

**json_sample_cli** is a command line tool for generating code either for one-time use or e.g. in a build script.

**json_sample_derive** is a library to generate code at compile time, by the way of a custom derive. It is probably more suited as a normal procedural macro, so this is something of a stopgap solution until normal procedural macros are part of stable Rust.
