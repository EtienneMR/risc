# Risc

A scripting language designed for gluing processes and transforming streams.

## Goals

**Simple to learn.** A minimal keyword set and one compound data type (the table) cover the vast majority of scripting needs. The language surface is intentionally small so it can be held in one's head without reference material.

**Stream-native.** Working with processes, files, and pipelines is a first-class concern rather than an afterthought. The pipe operator `|>` chains transformations left-to-right, the same way you would describe them in plain English.

**Loud failures.** Errors propagate explicitly. A function that can fail either raises (and callers must opt in to catching) or returns an error-table that the caller inspects. There is no silent null-coercion or type coercion; `"3" + 4` is a type error.

**Embedding-friendly.** Risc compiles to a single statically-linked binary with no runtime dependencies, making it straightforward to ship as part of a larger tool or drop onto a remote machine.

## Language sketch

```
-- Variables
let name  = "world"
let count = 42

-- Functions
let fn greet(name)
    "Hello, " + name + "!"
end

-- Tables (the one compound type)
let cfg = { .host = "localhost", .port = 8080 }
let arr = { "a", "b", "c" }    -- integer-keyed

-- Control flow
if count > 10 then
    print("big")
else
    print("small")
end

for item in iter.from(arr) do
    print(item)
end

for i in iter.range(10) do
    print(i)
end

-- Error handling
try
    risky_operation()
catch "io error" as e
    print("IO failed: " + e.message)
else
    print("success")
end

-- Pipe operator: left value becomes the first argument of the right call
fs.open("log.txt")
    |> iter.from
    |> iter.filter(fn(line) string.contains(line, "ERROR") end)
    |> iter.map(fn(line) string.trim(line) end)
    |> iter.collect
```

## Design decisions

**Tables as the only compound type.** Like Lua, a single flexible table type covers arrays (integer keys), dictionaries (string keys), and mixed structures. This keeps the runtime and the mental model small.

**Functions are values.** Functions close over their lexical environment and can be passed, returned, and stored in tables. There are no methods — member access is table indexing, and functions that operate on a table take it as their first argument.

**Iterators are zero-argument callables.** Anything callable with no arguments that returns a sequence of values ending in `nil` is an iterator. Generator functions, file readers, and range functions all share this protocol.

**The pipe operator is the primary composition tool.** `a |> f(b)` evaluates `f(b)` first (which for curried combinators returns a function) and then calls the result with `a`. This makes data-transformation pipelines readable without nesting.

## Standard library

| Module        | Purpose                                                                |
| ------------- | ---------------------------------------------------------------------- |
| `@std/iter`   | Lazy iterators: `from`, `range`, `map`, `filter`, `collect`, `fold`, … |
| `@std/table`  | Table utilities: `keys`, `values`, `items`, `from`, `clone`            |
| `@std/string` | Higher-level strings: `lines`, `words`, `join`, `indent`, `truncate`   |
| `@std/path`   | Path manipulation: `basename`, `dirname`, `ext`, `normalize`, `join`   |
| `@std/regex`  | Regex helpers: `scan_all`, `scan_group` + all `@core/regex` functions  |
| `@std/utf8`   | Unicode: `chars` iterator, `filter_chars`, `to_hex`, `is_ascii`        |
| `@std/json`   | File-level JSON: `load(path)`, `dump(path, value)`                     |
| `@std/log`    | Levelled logging: `info`, `warn`, `error`, `success`, `debug`, `die`   |
| `@std/cli`    | Declarative CLI argument parsing with `pos`, `flag`, `parse`           |
| `@std/exec`   | Process execution: `run`, `spawn`, `shell`, `shell_spawn`              |

The prelude (loaded automatically) exposes `string`, `iter`, and `table` as globals.

## Core library

Core modules are Rust implementations accessible via `require("@core/<name>")`.

| Module         | Purpose                                          |
| -------------- | ------------------------------------------------ |
| `@core/os`     | Filesystem, environment variables, process args  |
| `@core/exec`   | Subprocess execution (raw binary and shell)      |
| `@core/string` | String primitives (split, slice, pad, find, …)   |
| `@core/table`  | Table primitives (`keys`)                        |
| `@core/json`   | JSON parse and stringify                         |
| `@core/path`   | OS-aware path joining and canonicalisation       |
| `@core/regex`  | Regular expressions via regex-lite (RE2)         |
| `@core/utf8`   | UTF-8 encode/decode and codepoint utilities      |
| `@core/http`   | Synchronous HTTP client (GET, POST, PUT, DELETE) |

## Building

```sh
cargo build --release
```

The binary is fully statically linked (use the `x86_64-unknown-linux-musl` target on Linux for maximum portability). The stdlib is embedded into the binary at build time — no external files are needed at runtime.

## Running

```sh
# Start the REPL
risc

# Run a script
risc script.ri
```

## Error handling example

```
let exec = require("@std/exec")

let result = try
    exec.shell("git status")
catch "exec error" as e
    error("exec error", "git not found: " + e.message)
end

if result.code != 0 then
    print("not a git repo")
else
    print(result.stdout)
end
```
