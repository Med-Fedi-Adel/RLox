# rlox

A Rust implementation of [Lox](https://craftinginterpreters.com/), the language from [*Crafting Interpreters*](https://craftinginterpreters.com/) by Robert Nystrom.

This repository currently implements **Part II — A Tree-Walk Interpreter** (jlox). **Part III — A Bytecode Virtual Machine** (clox) is planned.

## Status

| Part | Name | Status |
|------|------|--------|
| II | A Tree-Walk Interpreter | Complete |
| III | A Bytecode Virtual Machine | Planned |

The tree-walk interpreter passes the official [Crafting Interpreters jlox test suite](https://github.com/munificent/craftinginterpreters#testing) (239 tests).

## Features

Everything from Part II of the book is implemented:

- Scanning, parsing, AST evaluation
- Variables, blocks, and scope
- Control flow: `if` / `else`, `while`, `for`, logical operators
- Functions, closures, and return
- Static resolution and indexed local storage
- Classes, instances, properties, `this`, constructors (`init`)
- Inheritance and `super`
- Native `clock()` function

### Extensions beyond the book

These are extra features rlox supports on top of standard Lox. They do not affect compatibility with the official jlox tests.

- `break` ; exit the innermost loop early
- Anonymous functions ; `fun (a, b) { ... }` as expressions
- Static methods ; `class Math { class square(n) { ... } }`
- Getters ; `area { return ... }` on instances

See `tests/tree_walk_interpreter.rlox` for a guided tour of the language.

## Requirements

- [Rust](https://www.rust-lang.org/) (2024 edition)
- [Dart](https://dart.dev/get-dart), only needed to run the official Crafting Interpreters test suite

## Quick start

```bash
# Build
cargo build --release

# REPL
./target/release/rlox

# Run a script
./target/release/rlox tests/tree_walk_interpreter.rlox

# Rust unit tests
cargo test
```

Or use the Makefile:

```bash
make help          # list all targets
make demo          # run the Part II summary program
make test          # cargo test
make test-official # official jlox conformance suite
```

## Makefile

| Target | Description |
|--------|-------------|
| `make` / `make help` | Show available targets |
| `make build` | Debug build |
| `make release` | Release build |
| `make run` | REPL (debug) |
| `make run-file FILE=path` | Run a `.rlox` script |
| `make demo` | Run `tests/tree_walk_interpreter.rlox` |
| `make test` | Run Rust unit tests |
| `make test-official` | Run the official jlox suite |
| `make test-official-quick` | Smoke-test one official case |
| `make check` | `cargo check` |
| `make clippy` | Run Clippy |
| `make fmt` | Format code |

Variables:

- `CRAFTING`: path to a clone of [craftinginterpreters](https://github.com/munificent/craftinginterpreters) (default: `~/Projects/craftinginterpreters`)
- `FILTER`: limit official tests to a path prefix, e.g. `FILTER=inheritance`

```bash
make test-official FILTER=super
make test-official CRAFTING=../craftinginterpreters
```

## Testing

### Rust unit tests

```bash
cargo test
```

Parser, scanner, resolver, and interpreter behavior are covered in `src/**/tests.rs`.

### Official jlox suite

Clone the book repository, install Dart dependencies, then:

```bash
git clone https://github.com/munificent/craftinginterpreters.git
cd craftinginterpreters/tool && dart pub get && cd ../..

make test-official
```

The Dart test runner expects a jlox-compatible interpreter. rlox uses the same exit codes as jlox:

- `65` — compile / resolution error
- `70` — runtime error

> **Note:** The book's test tooling targets Dart 2. If you are on Dart 3, you may need to update `tool/pubspec.yaml` in the craftinginterpreters clone (SDK constraint and dependencies).

### Local `.rlox` fixtures

The `tests/` directory contains hand-written programs used during development. They are not run automatically. Use `make run-file FILE=tests/foo.rlox`.

## Architecture

Execution follows the pipeline from the book:

```
source → Scanner → Parser → Resolver → Interpreter
```

| Module | Role |
|--------|------|
| `scanner` | Tokenize source text |
| `parser` | Build AST (`Expr`, `Stmt`) |
| `resolver` | Resolve variables, detect static errors |
| `interpreter` | Tree-walk evaluation |
| `environment` | Runtime scope chain with indexed locals |
| `expr_id` | Side table for resolved variable metadata |

### Indexed locals

Instead of looking up locals by name at runtime, the resolver assigns each local a `(distance, slot)` pair. The interpreter reads variables by index into a `Vec<Option<Value>>` per environment. See [indexed-local-storage.md](indexed-local-storage.md) for a write-up of this design.

## Project layout

```
src/
  main.rs           CLI entry point
  lox.rs            Driver (scan → parse → resolve → interpret)
  scanner/          Lexer
  parser/           Recursive-descent parser
  expr/             Expression AST
  stmt/             Statement AST
  resolver/         Static analysis
  interpreter/      Runtime (classes, functions, instances)
  environment.rs    Scope chain
  token.rs          Tokens and runtime values
tests/              Local .rlox fixtures and examples
tools/              AST generator (from the book)
```

## Roadmap

- [x] Part II — tree-walk interpreter (Chapters 7–13)
- [ ] Part III — bytecode virtual machine (Chapters 14–30)

When the bytecode VM lands, this README will be updated with clox architecture, opcodes, and a second test target (`make test-official-clox` or similar).

## References

- [Crafting Interpreters](https://craftinginterpreters.com/)
- [Book repository](https://github.com/munificent/craftinginterpreters)
- [Lox language summary](https://craftinginterpreters.com/appendix-i.html)

## License

This is full for learning purposes. The Lox language and test suite belong to the Crafting Interpreters project.
