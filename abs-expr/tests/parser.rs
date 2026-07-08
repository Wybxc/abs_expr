use abs_expr::{Expr, abs_expr};

// ============================================================
// Grammar: Primary — ident, literal, parenthesized
// ============================================================

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
fn deep_nested_atom() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!((x)), x);
}

// ============================================================
// Grammar: Prefix — op + expr(RHS at JUXTAPOSITION_BP)
// ============================================================

#[test]
fn prefix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(-x), Expr::Prefix { op: "-", expr: &x });
}

// --x as a single multi-char prefix operator
#[test]
fn double_dash_prefix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(--x), Expr::Prefix { op: "--", expr: &x });
}

// !- as a single Joint prefix operator
#[test]
fn double_prefix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(!-x), Expr::Prefix { op: "!-", expr: &x });
}

// ~~ as a single Joint prefix operator
#[test]
fn double_tilde_prefix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(~~x), Expr::Prefix { op: "~~", expr: &x });
}

// ++ as a single Joint prefix operator
#[test]
fn arbitrary_prefix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(++x), Expr::Prefix { op: "++", expr: &x });
}

// Parens disambiguate nested prefix: -(-x)
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
// Grammar: Postfix — expr + op (consumed after infix attempt)
// ============================================================

#[test]
fn postfix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(x!), Expr::Postfix { expr: &x, op: "!" });
}

// x!! as a single multi-char Joint postfix operator
#[test]
fn double_bang_postfix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(x!!), Expr::Postfix { expr: &x, op: "!!" });
}

// x! ! = (x!)! — space splits into two separate postfix ops
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

#[test]
fn arbitrary_postfix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!(x++), Expr::Postfix { expr: &x, op: "++" });
    assert_eq!(abs_expr!(x--), Expr::Postfix { expr: &x, op: "--" });
}

// ============================================================
// Grammar: Infix — expr op expr (precedence climbing)
// ============================================================
// --- Single-character infix operators ---

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

// --- Multi-character infix operators (Joint puncts, last-char precedence) ---

// ::  ends with : → cons (150)
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

// ->  ends with > → comparison (130)
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

// =>  ends with > → comparison (130)
#[test]
fn fat_arrow() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a => b),
        Expr::Infix {
            left: &a,
            op: "=>",
            right: &b
        }
    );
}

// <>  ends with > → comparison (130)
#[test]
fn diamond_operator() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a <> b),
        Expr::Infix {
            left: &a,
            op: "<>",
            right: &b
        }
    );
}

// <<  ends with < → comparison (130)
// >>  ends with > → comparison (130)
#[test]
fn shift_operators() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a << b),
        Expr::Infix {
            left: &a,
            op: "<<",
            right: &b
        }
    );
    assert_eq!(
        abs_expr!(a >> b),
        Expr::Infix {
            left: &a,
            op: ">>",
            right: &b
        }
    );
}

// ..  catch-all → comparison (130)
#[test]
fn range_operator() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a..b),
        Expr::Infix {
            left: &a,
            op: "..",
            right: &b
        }
    );
}

// ::: ends with : → cons (150)
#[test]
fn triple_colon_operator() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(
        abs_expr!(a ::: b),
        Expr::Infix {
            left: &a,
            op: ":::",
            right: &b
        }
    );
}

// --- Left associativity ---

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

// --- Precedence (tight → loose) ---

// * (170) > + (160)
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

// **  ends with * → multiplicative (170) > + (160)
#[test]
fn double_star_precedence() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let b_dstar_c = Expr::Infix {
        left: &b,
        op: "**",
        right: &c,
    };
    assert_eq!(
        abs_expr!(a + b ** c),
        Expr::Infix {
            left: &a,
            op: "+",
            right: &b_dstar_c
        }
    );
}

// -> ends with > → comparison (130) < + (160)
#[test]
fn arrow_looser_than_plus() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let a_plus_b = Expr::Infix {
        left: &a,
        op: "+",
        right: &b,
    };
    assert_eq!(
        abs_expr!(a + b -> c),
        Expr::Infix {
            left: &a_plus_b,
            op: "->",
            right: &c
        }
    );
}

// Full precedence: a + b * c - d / e = (a + (b * c)) - (d / e)
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

// Juxtaposition (185) > infix: a b + c d = (a b) + (c d)
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

// ============================================================
// Grammar: Juxtaposition — adjacent primary expressions
// ============================================================

#[test]
fn juxtaposition() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    assert_eq!(abs_expr!(a b), Expr::Juxtaposition(&[a, b]));
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

// ============================================================
// Parentheses: grouping, nesting, transparency
// ============================================================

// (a + b) * c  — parens override default precedence
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

// Parens + juxtaposition: a (b + c) ≠ a b + c
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

// (a b) c  — parens preserve inner juxtaposition grouping
#[test]
fn juxtaposition_of_grouped_expressions() {
    let a = Expr::Atom("a");
    let b = Expr::Atom("b");
    let c = Expr::Atom("c");
    let ab = Expr::Juxtaposition(&[a, b]);
    assert_eq!(abs_expr!((a b) c), Expr::Juxtaposition(&[ab, c]));
}

// -(a + b) — parens override prefix binding
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

// (x)! — parens then postfix
#[test]
fn parentheses_with_postfix() {
    let x = Expr::Atom("x");
    assert_eq!(abs_expr!((x)!), Expr::Postfix { expr: &x, op: "!" });
}

// (f x)! — juxtaposition inside parens then postfix
#[test]
fn grouped_then_postfix() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let fx = Expr::Juxtaposition(&[f, x]);
    assert_eq!(abs_expr!((f x)!), Expr::Postfix { expr: &fx, op: "!" });
}

// ============================================================
// Interactions: Prefix × Juxtaposition
// Prefix RHS parsed at JUXTAPOSITION_BP, so -f x = -(f x)
// ============================================================

#[test]
fn prefix_juxtaposes_in_rhs() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let fx = Expr::Juxtaposition(&[f, x]);
    assert_eq!(abs_expr!(-f x), Expr::Prefix { op: "-", expr: &fx });
}

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
// Interactions: Postfix × Juxtaposition
// Postfix binds before juxtaposition: f x! = f (x!)
// When op is both postfix and infix and a primary follows → infix wins
// ============================================================

#[test]
fn postfix_before_juxtaposition() {
    let f = Expr::Atom("f");
    let x = Expr::Atom("x");
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(abs_expr!(f x!), Expr::Juxtaposition(&[f, x_fact]));
}

#[test]
fn postfix_after_juxtaposition_chain() {
    let f = Expr::Atom("f");
    let g = Expr::Atom("g");
    let x = Expr::Atom("x");
    let fg = Expr::Juxtaposition(&[f, g]);
    let x_fact = Expr::Postfix { expr: &x, op: "!" };
    assert_eq!(abs_expr!(f g x!), Expr::Juxtaposition(&[fg, x_fact]));
}

// x! y → infix wins over postfix+juxtaposition
#[test]
fn postfix_then_juxtaposition() {
    let x = Expr::Atom("x");
    let y = Expr::Atom("y");
    assert_eq!(
        abs_expr!(x! y),
        Expr::Infix {
            left: &x,
            op: "!",
            right: &y
        }
    );
}

// f! g! → infix wins over postfix+juxtaposition
#[test]
fn both_postfix_in_juxtaposition() {
    let f = Expr::Atom("f");
    let g = Expr::Atom("g");
    let g_fact = Expr::Postfix { expr: &g, op: "!" };
    assert_eq!(
        abs_expr!(f! g!),
        Expr::Infix {
            left: &f,
            op: "!",
            right: &g_fact
        }
    );
}

// x! y! → infix wins over postfix+juxtaposition
#[test]
fn both_sides_postfix_in_juxtaposition() {
    let x = Expr::Atom("x");
    let y = Expr::Atom("y");
    let y_fact = Expr::Postfix { expr: &y, op: "!" };
    assert_eq!(
        abs_expr!(x! y!),
        Expr::Infix {
            left: &x,
            op: "!",
            right: &y_fact
        }
    );
}

// ============================================================
// Interactions: Prefix × Postfix
// Postfix binds first, then prefix: -x! = -(x!)
// ============================================================

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

// ============================================================
// Interactions: Postfix × Infix
// Postfix binds before infix: x! + y = (x!) + y
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

// f x! + g y! * z = ((f (x!)) + ((g (y!)) * z))
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

// (f x)! + y — parens force juxtaposition before postfix
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
