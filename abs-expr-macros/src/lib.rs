enum Expr {
    Atom(String),
    Juxtaposition(Vec<Expr>),
    Prefix {
        op: String,
        expr: Box<Expr>,
    },
    Postfix {
        expr: Box<Expr>,
        op: String,
    },
    Infix {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
}

impl unsynn::Parser for Expr {
    fn parser(tokens: &mut unsynn::TokenIter) -> unsynn::Result<Self> {
        todo!()
    }
}

#[proc_macro]
pub fn abs_expr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    todo!()
}
