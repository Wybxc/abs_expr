#[cfg(test)]
mod tests {
    #[test]
    fn atom_ident() {
        let s = abs_expr_macros::abs_expr!(x);
        assert_eq!(s, "Atom(\"x\")");
    }

    #[test]
    fn atom_literal() {
        let s = abs_expr_macros::abs_expr!(42);
        assert_eq!(s, "Atom(\"42\")");
    }

    #[test]
    fn prefix() {
        let s = abs_expr_macros::abs_expr!(-x);
        assert_eq!(s, "Prefix { op: \"-\", expr: Atom(\"x\") }");
    }

    #[test]
    fn postfix() {
        let s = abs_expr_macros::abs_expr!(x!);
        assert_eq!(s, "Postfix { expr: Atom(\"x\"), op: \"!\" }");
    }

    #[test]
    fn infix_plus() {
        let s = abs_expr_macros::abs_expr!(a + b);
        assert_eq!(
            s,
            "Infix { left: Atom(\"a\"), op: \"+\", right: Atom(\"b\") }"
        );
    }

    #[test]
    fn infix_mul() {
        let s = abs_expr_macros::abs_expr!(a * b);
        assert_eq!(
            s,
            "Infix { left: Atom(\"a\"), op: \"*\", right: Atom(\"b\") }"
        );
    }

    #[test]
    fn juxtaposition() {
        let s = abs_expr_macros::abs_expr!(a b);
        assert_eq!(s, "Juxtaposition([Atom(\"a\"), Atom(\"b\")])");
    }

    #[test]
    fn parentheses() {
        let s = abs_expr_macros::abs_expr!((a + b) * c);
        assert_eq!(
            s,
            "Infix { left: Infix { left: Atom(\"a\"), op: \"+\", right: Atom(\"b\") }, op: \"*\", right: Atom(\"c\") }"
        );
    }

    #[test]
    fn precedence_mul_over_add() {
        let s = abs_expr_macros::abs_expr!(a + b * c);
        assert_eq!(
            s,
            "Infix { left: Atom(\"a\"), op: \"+\", right: Infix { left: Atom(\"b\"), op: \"*\", right: Atom(\"c\") } }"
        );
    }

    #[test]
    fn juxtaposition_over_infix() {
        let s = abs_expr_macros::abs_expr!(a b + c d);
        assert_eq!(
            s,
            "Infix { left: Juxtaposition([Atom(\"a\"), Atom(\"b\")]), op: \"+\", right: Juxtaposition([Atom(\"c\"), Atom(\"d\")]) }"
        );
    }

    #[test]
    fn double_infix() {
        let s = abs_expr_macros::abs_expr!(a + b + c);
        assert_eq!(
            s,
            "Infix { left: Infix { left: Atom(\"a\"), op: \"+\", right: Atom(\"b\") }, op: \"+\", right: Atom(\"c\") }"
        );
    }

    #[test]
    fn compound_operator() {
        let s = abs_expr_macros::abs_expr!(a::b);
        assert_eq!(
            s,
            "Infix { left: Atom(\"a\"), op: \"::\", right: Atom(\"b\") }"
        );
    }

    #[test]
    fn arrow_operator() {
        let s = abs_expr_macros::abs_expr!(a -> b);
        assert_eq!(
            s,
            "Infix { left: Atom(\"a\"), op: \"->\", right: Atom(\"b\") }"
        );
    }

    // Right-associative: a ** b ** c → a ** (b ** c)
    #[test]
    fn right_assoc_exponentiation() {
        let s = abs_expr_macros::abs_expr!(a**b**c);
        assert_eq!(
            s,
            "Infix { left: Atom(\"a\"), op: \"**\", right: Infix { left: Atom(\"b\"), op: \"**\", right: Atom(\"c\") } }"
        );
    }

    // Left-associative: a - b - c → (a - b) - c
    #[test]
    fn left_assoc_sub() {
        let s = abs_expr_macros::abs_expr!(a - b - c);
        assert_eq!(
            s,
            "Infix { left: Infix { left: Atom(\"a\"), op: \"-\", right: Atom(\"b\") }, op: \"-\", right: Atom(\"c\") }"
        );
    }

    // Mixed precedence
    #[test]
    fn mixed_precedence() {
        let s = abs_expr_macros::abs_expr!(a + b * c - d / e);
        assert_eq!(
            s,
            "Infix { left: Infix { left: Atom(\"a\"), op: \"+\", right: Infix { left: Atom(\"b\"), op: \"*\", right: Atom(\"c\") } }, op: \"-\", right: Infix { left: Atom(\"d\"), op: \"/\", right: Atom(\"e\") } }"
        );
    }
}
