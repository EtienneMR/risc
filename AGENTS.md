# AGENTS.md for Risc

This document provides high-signal guidance for OpenCode agents working on the Risc project.

## Project Overview

Risc is a scripting language implemented in Rust, designed for gluing processes and transforming streams.

## Project Structure

- `src/`: Main Rust implementation.
  - `src/corelib/`: Core modules accessible via `require("@core/<name>")`.
  - `src/parser/`: Parsing logic.
- `stdlib/`: Standard library implementation in risc.
  - `build.rs` bundle them for `runtime::stdlib` module.
- `tests/`: integration tests written in risc, it is the primary source of truth for language features and usage.
  - `build.rs` bundle them for `main::tests` module.

## Development Commands

- **Build Release Binary**:

  ```sh
  cargo build --release
  ```

  For maximum portability on Linux, target `x86_64-unknown-linux-musl`. The stdlib is embedded in the binary.

- **Run All Tests**:

  ```sh
  cargo test --locked
  ```

- **Start REPL**:

  ```sh
  risc
  ```

- **Run a Risc Script**:
  ```sh
  risc script.ri
  ```

## Coding Conventions

- **File Documentation**: Every source file in `src/` _must_ begin with a 5-line `//!` comment block that succinctly summarizes the file's purpose and functionality.

## CI/CD Workflow Quirks

- **Release Gating**: Releases are skipped if the commit message contains "WIP" (case-insensitive).
- **Platform Builds**: CI builds for Linux (x86_64-unknown-linux-musl), macOS (aarch64-apple-darwin), and Windows (x86_64-pc-windows-msvc).
- **Testing Environment**: `cargo test --locked` is currently only run on the `ubuntu-latest` CI runner.
- **Release Tag Format**: GitHub release tags are `v<Cargo.toml_version>-<short_commit_sha>`.
- **Release Notes**: The commit message is used directly as the release notes.
