# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/Wybxc/abs_expr/compare/abs-expr-v0.1.0...abs-expr-v0.1.1) - 2026-07-08

### Other

- add BNF grammar to README, reorganize tests by grammar
- inline trivial prefix/postfix helpers, deduplicate primary check
- accept arbitrary prefix/postfix operators, prefer infix over postfix+juxt
- use last-char precedence for operators, remove char whitelist
