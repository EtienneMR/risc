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
let name = "world"
let count = 42

-- Functions
let greet = fn(name)
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

for item in collection do
    print(item)
end

while condition do
    ...
end

-- Error handling
let result = try
    risky_operation()
catch "IOError" as e
    print("IO failed: " + e.msg)
else
    print("success")
end

-- Pipe operator
fs.open("log.txt")
    |> fs.bytes
    |> toUTF8
    |> filter(fn(line) contains(line, "ERROR") end)
    |> each(print)
```

## Design decisions

**Tables as the only compound type.** Like Lua, a single flexible table type covers arrays (integer keys), dictionaries (string keys), and mixed structures. This keeps the runtime and the mental model small.

**Functions are values.** Functions close over their lexical environment and can be passed, returned, and stored in tables. There are no methods — member access is table indexing, and functions that operate on a table take it as their first argument.

**Iterators are zero-argument callables.** Anything callable with no arguments that returns a sequence of values ending in `nil` is an iterator. Generator functions, file readers, and range functions all share this protocol.

**Errors are tables.** A raised error is a table with at least an `error` (kind) field and a `message` field. Catch arms match on the kind. Uncaught errors unwind to the top level and are displayed with source context.

**The pipe operator is the primary composition tool.** `a |> f(b)` is equivalent to `f(a, b)` — the left-hand value is injected as the first argument of the right-hand call. This makes data-transformation pipelines readable without nesting.
