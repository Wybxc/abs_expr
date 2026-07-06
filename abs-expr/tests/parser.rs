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
