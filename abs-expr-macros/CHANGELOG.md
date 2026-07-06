# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/Wybxc/abs_expr/releases/tag/abs-expr-macros-v0.1.0) - 2026-07-06

### Added

- redesign postfix binding and prefix RHS for intuitive operator precedence
- add Grouped variant to preserve parentheses in juxtaposition
- add heap-free Expr type and static code generation
- simplify macro usage in tests by removing redundant namespace
- implement OCaml-like precedence parser for Expr using unsynn
- define Expr AST and add unsynn parser skeleton
- init

### Other

- unify operator reading, add infix char filter, use compile_error
- clean up comments and dead_code attr in macros lib
- inline temporary clone variables in peek checks
- use const block for code generation output
- fix clippy warnings and workspace resolver
