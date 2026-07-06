/// A heap-allocation-free expression tree for statically generated expressions.
///
/// All data is stored via references (`&'a str`, `&'a [Expr<'a>]`, `&'a Expr<'a>`),
/// so the entire tree can live in static memory with no heap allocation.
///
/// Constructed at compile time via the `abs_expr!` macro:
/// ```
/// use abs_expr::{Expr, abs_expr};
/// let expr: Expr<'static> = abs_expr!(a + b * c);
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Expr<'a> {
    Atom(&'a str),
    Juxtaposition(&'a [Expr<'a>]),
    Prefix {
        op: &'a str,
        expr: &'a Expr<'a>,
    },
    Postfix {
        expr: &'a Expr<'a>,
        op: &'a str,
    },
    Infix {
        left: &'a Expr<'a>,
        op: &'a str,
        right: &'a Expr<'a>,
    },
}

pub use abs_expr_macros::abs_expr;
