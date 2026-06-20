# json-parser-rs

This is a small JSON parser I wrote in Rust to learn more about how lexers and parsers work under the hood instead of just using `serde_json`.

It takes a raw JSON string, turns it into a list of tokens, and then builds a simple tree (AST) out of it.

## Features

- **Lexer**: Breaks down input into tokens like numbers, strings, booleans (`true`/`false`), `null`, brackets, braces, colons, and commas.
- **Escape sequences**: Handles common string escapes like `\n`, `\t`, `\"`, `\\`, and 4-digit unicode like `\u0041`.
- **Parser**: Turns the tokens into `JsonValue` types (Objects, Arrays, Strings, Numbers, Bools, Null).
- **Printer**: Has a simple `display()` function to turn `JsonValue` back into a formatted string.

## Known Limitations

- **Line & column numbers**: If there's a syntax error, it tells you *what* went wrong, but not the exact line number where it happened.
- **Strict spec rules**: Number parsing just uses standard `f64::parse`, so it might accept a few number formats that standard JSON technically forbids (like leading zeros).
- **Performance**: I used basic `Vec` allocations everywhere, so it's not super fast or memory-optimized.


## Running Tests

Run the unit test suite covering the lexer, parser, and printer:

```bash
cargo test

```
