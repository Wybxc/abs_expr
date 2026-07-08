# abs-expr

A heap-allocation-free expression tree for statically generated DSL expressions.

Expressions are constructed at compile time via the `abs_expr!` macro:

```rust
use abs_expr::{Expr, abs_expr};

let expr: Expr<'static> = abs_expr!(a + b * c);
```

All data lives in `const` memory — zero heap allocation at runtime.

## Grammar

```text
expr        → semi-expr

(* Precedence levels, lowest to highest *)

semi-expr   → comma-expr (";" comma-expr)?               (* 80  *)
comma-expr  → comp-expr ("," comp-expr)*                 (* 100 *)
comp-expr   → concat-expr (cop concat-expr)*             (* 130 *)
concat-expr → cons-expr (cat-op cons-expr)?              (* 140 *)
cons-expr   → add-expr (cons-op add-expr)?               (* 150 *)
add-expr    → mul-expr (add-op mul-expr)*                (* 160 *)
mul-expr    → juxt-expr (mul-op juxt-expr)*              (* 170 *)

juxt-expr   → postfix-expr+                              (* 185 *)

postfix-expr → prefix-expr
             | postfix-expr pop                           (* postfix *)
prefix-expr → primary
             | pop prefix-expr                            (* prefix *)

primary     → ident
             | literal
             | "(" expr ")"

(* Operator classification — last character determines precedence *)
cop         → op-seq where last-char ∈ {= < > | & $ # ! ? ~}
            | op-seq                                      (* catch-all *)
cat-op      → op-seq where last-char ∈ {@ ^}
cons-op     → op-seq where last-char ∈ {:}
add-op      → op-seq where last-char ∈ {+ -}
mul-op      → op-seq where last-char ∈ {* / %}
pop         → op-seq

op-seq      → punct (joint-punct)*
```

### Disambiguation rules

1. **Infix wins**: When a postfix operator is followed by a primary expression and the operator is also a valid infix operator, it parses as infix rather than postfix+juxtaposition.
2. **Prefix includes juxtaposition**: The right-hand side of a prefix operator is parsed at the juxtaposition precedence level.
3. **Parentheses are transparent**: Parentheses only affect grouping during parsing; they do not produce nodes in the final AST.
4. **Last-character precedence**: Multi-character operator precedence is determined by the operator's last character (e.g., `->` ends with `>` → comparison level 130).

### Precedence table

|Level|Associativity|Last char|Examples|
|---|---|---|---|
|80|non-assoc|`;`|`;`|
|100|left|`,`|`,`|
|130|left|`= < > \| & $ # ! ? ~`|`->` `=>` `<>` `<<` `>>`|
|140|non-assoc|`@ ^`|`@` `^`|
|150|non-assoc|`:`|`::` `:::`|
|160|left|`+ -`|`+` `-` `++` `--`|
|170|left|`* / %`|`*` `/` `**` `//`|
|185|left|—|juxtaposition `a b`|
|—|—|—|prefix `-x`, postfix `x!`|

## License

Licensed under the Apache License, Version 2.0.
