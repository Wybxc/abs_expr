use abs_expr::{Expr, abs_expr};

#[test]
fn atom_ident() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(x), x);
}

#[test]
fn atom_literal() {
    assert_eq!(abs_expr!(42), Expr::Atom("42"));
}

#[test]
fn prefix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(-x), Expr::Prefix { op: "-", expr: &x });
}

#[test]
fn postfix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(x!), Expr::Postfix { expr: &x, op: "!" });
}

#[test]
fn infix_plus() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a + b),
        Expr::Infix {
            left: &a,
            op: "+",
            right: &b
        }
    );
}

#[test]
fn infix_mul() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a * b),
        Expr::Infix {
            left: &a,
            op: "*",
            right: &b
        }
    );
}

#[test]
fn juxtaposition() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(abs_expr!(a b), Expr::Juxtaposition(&[a, b]));
}

#[test]
fn parentheses() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let a_plus_b = Expr::Infix {
        left: &a,
        op: "+",
        right: &b,
    };
    assert_eq!(
        abs_expr!((a + b) * c),
        Expr::Infix {
            left: &a_plus_b,
            op: "*",
            right: &c
        }
    );
}

#[test]
fn precedence_mul_over_add() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let b_mul_c = Expr::Infix {
        left: &b,
        op: "*",
        right: &c,
    };
    assert_eq!(
        abs_expr!(a + b * c),
        Expr::Infix {
            left: &a,
            op: "+",
            right: &b_mul_c
        }
    );
}

#[test]
fn juxtaposition_over_infix() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let d = Expr::Atom("d");
    let ab = Expr::Juxtaposition(&[a, b]);
    let cd = Expr::Juxtaposition(&[c, d]);
    assert_eq!(
        abs_expr!(a b + c d),
        Expr::Infix {
            left: &ab,
            op: "+",
            right: &cd
        }
    );
}

#[test]
fn double_infix() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let a_plus_b = Expr::Infix {
        left: &a,
        op: "+",
        right: &b,
    };
    assert_eq!(
        abs_expr!(a + b + c),
        Expr::Infix {
            left: &a_plus_b,
            op: "+",
            right: &c
        }
    );
}

#[test]
fn compound_operator() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a::b),
        Expr::Infix {
            left: &a,
            op: "::",
            right: &b
        }
    );
}

#[test]
fn arrow_operator() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a -> b),
        Expr::Infix {
            left: &a,
            op: "->",
            right: &b
        }
    );
}

#[test]
fn left_assoc_sub() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let a_sub_b = Expr::Infix {
        left: &a,
        op: "-",
        right: &b,
    };
    assert_eq!(
        abs_expr!(a - b - c),
        Expr::Infix {
            left: &a_sub_b,
            op: "-",
            right: &c
        }
    );
}

#[test]
fn mixed_precedence() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let d = Expr::Atom("d");
    let e = Expr::Atom("e");
    let b_mul_c = Expr::Infix {
        left: &b,
        op: "*",
        right: &c,
    };
    let d_div_e = Expr::Infix {
        left: &d,
        op: "/",
        right: &e,
    };
    let a_plus_b_mul_c = Expr::Infix {
        left: &a,
        op: "+",
        right: &b_mul_c,
    };
    assert_eq!(
        abs_expr!(a + b * c - d / e),
        Expr::Infix {
            left: &a_plus_b_mul_c,
            op: "-",
            right: &d_div_e
        }
    );
}

#[test]
fn nested_parentheses() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let a_plus_b = Expr::Infix {
        left: &a,
        op: "+",
        right: &b,
    };
    assert_eq!(
        abs_expr!(((a + b) * c)),
        Expr::Infix {
            left: &a_plus_b,
            op: "*",
            right: &c
        }
    );
}

#[test]
fn parentheses_both_sides() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let d = Expr::Atom("d");
    let a_plus_b = Expr::Infix {
        left: &a,
        op: "+",
        right: &b,
    };
    let c_plus_d = Expr::Infix {
        left: &c,
        op: "+",
        right: &d,
    };
    assert_eq!(
        abs_expr!((a + b) * (c + d)),
        Expr::Infix {
            left: &a_plus_b,
            op: "*",
            right: &c_plus_d
        }
    );
}

#[test]
fn parentheses_with_juxtaposition() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let b_plus_c = Expr::Infix {
        left: &b,
        op: "+",
        right: &c,
    };
    assert_eq!(abs_expr!(a(b + c)), Expr::Juxtaposition(&[a, b_plus_c]));
}

#[test]
fn parentheses_with_prefix() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let a_plus_b = Expr::Infix {
        left: &a,
        op: "+",
        right: &b,
    };
    assert_eq!(
        abs_expr!(-(a + b)),
        Expr::Prefix {
            op: "-",
            expr: &a_plus_b
        }
    );
}

#[test]
fn parentheses_with_postfix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!((x)!), Expr::Postfix { expr: &x, op: "!" });
}

#[test]
fn deep_nested_atom() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!((x)), x);
}

#[test]
fn juxtaposition_of_grouped_expressions() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let ab = Expr::Juxtaposition(&[a, b]);
    // Parentheses preserve grouping: (a b) c ≠ a b c
    assert_eq!(abs_expr!((a b) c), Expr::Juxtaposition(&[ab, c]));
}

#[test]
fn chained_juxtaposition() {
    let f = Expr::Atom("f");
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let d = Expr::Atom("d");
    assert_eq!(abs_expr!(f a b c d), Expr::Juxtaposition(&[f, a, b, c, d]));
}

#[test]
fn complex_nested_parentheses() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let d = Expr::Atom("d");
    let e = Expr::Atom("e");
    let a_plus_b = Expr::Infix {
        left: &a,
        op: "+",
        right: &b,
    };
    let c_sub_d = Expr::Infix {
        left: &c,
        op: "-",
        right: &d,
    };
    let left_group = Expr::Infix {
        left: &a_plus_b,
        op: "*",
        right: &c_sub_d,
    };
    assert_eq!(
        abs_expr!(((a + b) * (c - d)) / e),
        Expr::Infix {
            left: &left_group,
            op: "/",
            right: &e
        }
    );
}

// ============================================================
// Postfix binding tests
// ============================================================

// Postfix binds to the immediately preceding primary, BEFORE juxtaposition.
// f x!  = f (x!)  — NOT (f x)!
#[test]
fn postfix_before_juxtaposition() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(abs_expr!(f x!), Expr::Juxtaposition(&[f, x_fact]));
}

// Postfix after a juxtaposition chain: f g x! = ((f g) (x!))
#[test]
fn postfix_after_juxtaposition_chain() {
    let f = Expr::Atom("f");
    let g = Expr::Atom("g");
    let x = Expr::Atom("x");
    let fg = Expr::Juxtaposition(&[f, g]);
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(abs_expr!(f g x!), Expr::Juxtaposition(&[fg, x_fact]));
}

// Parentheses force grouping before postfix: (f x)! ≠ f x!
#[test]
fn grouped_then_postfix() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let fx = Expr::Juxtaposition(&[f, x]);
    assert_eq!(abs_expr!((f x)!), Expr::Postfix { expr: &fx, op: "!" });
}

// Both sides have postfix: f! g! = ((f!) (g!))
#[test]
fn both_postfix_in_juxtaposition() {
    let f = Expr::Atom("f");
    let g = Expr::Atom("g");
    let f_fact = Expr::Postfix { expr: &f, op: "!" };
    let g_fact = Expr::Postfix { expr: &g, op: "!" };
    assert_eq!(abs_expr!(f! g!), Expr::Juxtaposition(&[f_fact, g_fact]));
}

// ============================================================
// Prefix + Juxtaposition interaction
// ============================================================

// Prefix RHS includes juxtaposition: -f x = -(f x)
#[test]
fn prefix_juxtaposes_in_rhs() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let fx = Expr::Juxtaposition(&[f, x]);
    assert_eq!(abs_expr!(-f x), Expr::Prefix { op: "-", expr: &fx });
}

// Prefix RHS includes juxtaposition with postfix: -f x! = -(f (x!))
#[test]
fn prefix_with_postfix_in_rhs() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    let fx_fact = Expr::Juxtaposition(&[f, x_fact]);
    assert_eq!(
        abs_expr!(-f x!),
        Expr::Prefix {
            op: "-",
            expr: &fx_fact
        }
    );
}

// ============================================================
// Prefix + Postfix interaction
// ============================================================

// -x! = -(x!) — postfix binds first, then prefix
#[test]
fn prefix_then_postfix() {
    let x = Expr::Atom("x");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(
        abs_expr!(-x!),
        Expr::Prefix {
            op: "-",
            expr: &x_fact
        }
    );
}

// !-x = !(-x) — two prefix operators
#[test]
fn double_prefix() {
    let x = Expr::Atom("x");
    let neg_x = Expr::Prefix { op: "-", expr: &x };
    assert_eq!(
        abs_expr!(!-x),
        Expr::Prefix {
            op: "!",
            expr: &neg_x
        }
    );
}

#[test]
fn separate_prefix_ops_via_parens() {
    let x = Expr::Atom("x");
    let neg_x = Expr::Prefix { op: "-", expr: &x };
    assert_eq!(
        abs_expr!(-(-x)),
        Expr::Prefix {
            op: "-",
            expr: &neg_x
        }
    );
}

// ============================================================
// Postfix + Infix interaction
// ============================================================

#[test]
fn postfix_then_infix() {
    let x = Expr::Atom("x");
    let y = Expr::Atom("y");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(
        abs_expr!(x! + y),
        Expr::Infix {
            left: &x_fact,
            op: "+",
            right: &y
        }
    );
}

#[test]
fn postfix_then_infix_sub() {
    let x = Expr::Atom("x");
    let y = Expr::Atom("y");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(
        abs_expr!(x! - y),
        Expr::Infix {
            left: &x_fact,
            op: "-",
            right: &y
        }
    );
}

// ============================================================
// Postfix + Juxtaposition interaction
// ============================================================

#[test]
fn postfix_then_juxtaposition() {
    let x = Expr::Atom("x");
    let y = Expr::Atom("y");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(abs_expr!(x! y), Expr::Juxtaposition(&[x_fact, y]));
}

#[test]
fn both_sides_postfix_in_juxtaposition() {
    let x = Expr::Atom("x");
    let y = Expr::Atom("y");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    let y_fact = Expr::Postfix { expr: &y, op: "!" };
    assert_eq!(abs_expr!(x! y!), Expr::Juxtaposition(&[x_fact, y_fact]));
}

// ============================================================
// Multi-char postfix operators
// ============================================================

// x!! = Postfix(x, "!!") — single multi-char operator
#[test]
fn double_bang_postfix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(x!!), Expr::Postfix { expr: &x, op: "!!" });
}

// x! ! = (x!)! — two separate applications (space splits them)
#[test]
fn chained_postfix_separate() {
    let x = Expr::Atom("x");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(
        abs_expr!(x! !),
        Expr::Postfix {
            expr: &x_fact,
            op: "!"
        }
    );
}

// ============================================================
// Complex combinations
// ============================================================

// -x! + y = (-(x!)) + y
#[test]
fn prefix_postfix_infix_combined() {
    let x = Expr::Atom("x");
    let y = Expr::Atom("y");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    let neg_x_fact = Expr::Prefix {
        op: "-",
        expr: &x_fact,
    };
    assert_eq!(
        abs_expr!(-x! + y),
        Expr::Infix {
            left: &neg_x_fact,
            op: "+",
            right: &y
        }
    );
}

// f x! + g y! * z
// = ((f (x!)) + ((g (y!)) * z))
#[test]
fn complex_postfix_infix_juxtaposition() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let g = Expr::Atom("g");
    let y = Expr::Atom("y");
    let z = Expr::Atom("z");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    let y_fact = Expr::Postfix { expr: &y, op: "!" };
    let fx = Expr::Juxtaposition(&[f, x_fact]);
    let gy = Expr::Juxtaposition(&[g, y_fact]);
    let gy_mul_z = Expr::Infix {
        left: &gy,
        op: "*",
        right: &z,
    };
    assert_eq!(
        abs_expr!(f x! + g y! * z),
        Expr::Infix {
            left: &fx,
            op: "+",
            right: &gy_mul_z
        }
    );
}

// (f x)! + y  — parentheses force grouping, different from f x! + y
#[test]
fn verify_parens_changes_meaning() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let y = Expr::Atom("y");
    let fx = Expr::Juxtaposition(&[f, x]);
    let fx_fact = Expr::Postfix { expr: &fx, op: "!" };
    assert_eq!(
        abs_expr!((f x)! + y),
        Expr::Infix {
            left: &fx_fact,
            op: "+",
            right: &y
        }
    );
}

// --x (single multi-char prefix operator)
#[test]
fn double_dash_prefix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(--x), Expr::Prefix { op: "--", expr: &x });
}
