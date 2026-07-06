use proc_macro2::{Delimiter, Ident, Spacing, TokenStream, TokenTree};
use quote::quote;
use unsynn::*;

#[derive(Debug)]
enum ParsedExpr {
    Atom(String),
    Juxtaposition(Vec<ParsedExpr>),
    Grouped(Box<ParsedExpr>),
    Prefix {
        op: String,
        expr: Box<ParsedExpr>,
    },
    Postfix {
        expr: Box<ParsedExpr>,
        op: String,
    },
    Infix {
        left: Box<ParsedExpr>,
        op: String,
        right: Box<ParsedExpr>,
    },
}

// Operator precedence levels (higher = tighter binding)
const PREFIX_BP: u32 = 210;
const JUXTAPOSITION_BP: u32 = 185;
const MULTIPLICATIVE_BP: u32 = 170; // * / %
const ADDITIVE_BP: u32 = 160; // + -
const CONS_BP: u32 = 150; // :
const CONCAT_BP: u32 = 140; // @ ^
const COMPARISON_BP: u32 = 130; // = < > | & $ #
const COMMA_BP: u32 = 100; // ,
const SEMICOLON_BP: u32 = 80; // ;

/// Read Joint Puncts incrementally, returning the longest operator string
/// that satisfies `is_valid`. On success, tokens are positioned after the
/// matched operator. On failure, tokens are restored to their original position.
fn try_read_longest_op<F>(tokens: &mut TokenIter, is_valid: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    let snapshot = tokens.clone();
    let mut op = String::new();
    let mut best: Option<(String, TokenIter)> = None;

    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(p) => {
                op.push(p.as_char());
                if is_valid(&op) {
                    best = Some((op.clone(), tokens.clone()));
                }
                if p.spacing() != Spacing::Joint {
                    break;
                }
            }
            _ => break,
        }
    }

    match best {
        Some((matched_op, pos)) => {
            *tokens = pos;
            Some(matched_op)
        }
        None => {
            *tokens = snapshot;
            None
        }
    }
}

/// Read a full operator (sequence of joint puncts) from the token stream.
fn read_operator(tokens: &mut TokenIter) -> Option<String> {
    let mut op = String::new();
    match tokens.next()? {
        TokenTree::Punct(p) => {
            op.push(p.as_char());
            let mut spacing = p.spacing();
            while spacing == Spacing::Joint {
                match tokens.next()? {
                    TokenTree::Punct(next) => {
                        op.push(next.as_char());
                        spacing = next.spacing();
                    }
                    _ => return None,
                }
            }
            Some(op)
        }
        _ => None,
    }
}

/// Get left and right binding power for an infix operator.
fn infix_bp(op: &str) -> Option<(u32, u32)> {
    let first = op.chars().next()?;
    match first {
        '*' | '/' | '%' => Some((MULTIPLICATIVE_BP, MULTIPLICATIVE_BP + 1)),
        '+' | '-' => Some((ADDITIVE_BP, ADDITIVE_BP + 1)),
        ':' => Some((CONS_BP, CONS_BP)),
        '@' | '^' => Some((CONCAT_BP, CONCAT_BP)),
        '=' | '<' | '>' | '|' | '&' | '$' | '#' => Some((COMPARISON_BP, COMPARISON_BP + 1)),
        ',' => Some((COMMA_BP, COMMA_BP + 1)),
        ';' => Some((SEMICOLON_BP, SEMICOLON_BP)),
        _ => None,
    }
}

/// Get binding power for a prefix operator.
fn prefix_bp(op: &str) -> Option<u32> {
    match op {
        "!" | "?" | "~" | "-" | "-." | "--" => Some(PREFIX_BP),
        _ => None,
    }
}

/// Get binding power for a postfix operator.
fn postfix_bp(op: &str) -> Option<u32> {
    match op {
        "!" | "!!" => Some(PREFIX_BP),
        _ => None,
    }
}

/// Check if the next token in the stream starts an expression
/// (including prefix operators).
fn peek_is_expr_start(tokens: &mut TokenIter) -> bool {
    let mut clone = tokens.clone();
    match clone.next() {
        Some(TokenTree::Ident(_)) | Some(TokenTree::Literal(_)) => true,
        Some(TokenTree::Group(g)) => g.delimiter() == Delimiter::Parenthesis,
        Some(TokenTree::Punct(p)) => matches!(p.as_char(), '!' | '?' | '~' | '-'),
        _ => false,
    }
}

/// Check if the next token in the stream starts a primary expression
/// (atom, literal, or parenthesized group) — excludes prefix operators.
fn peek_is_primary_start(tokens: &mut TokenIter) -> bool {
    let mut clone = tokens.clone();
    match clone.next() {
        Some(TokenTree::Ident(_)) | Some(TokenTree::Literal(_)) => true,
        Some(TokenTree::Group(g)) => g.delimiter() == Delimiter::Parenthesis,
        _ => false,
    }
}

/// Parse a primary expression: identifier, literal, or parenthesized expression.
#[allow(clippy::result_large_err)]
fn parse_primary(tokens: &mut TokenIter) -> Result<ParsedExpr> {
    match tokens.next() {
        Some(TokenTree::Ident(ident)) => Ok(ParsedExpr::Atom(ident.to_string())),
        Some(TokenTree::Literal(lit)) => Ok(ParsedExpr::Atom(lit.to_string())),
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            let mut inner = TokenIter::new(group.stream());
            let expr = parse_expr(&mut inner, 0)?;
            Ok(ParsedExpr::Grouped(Box::new(expr)))
        }
        _ => Error::unexpected_token(None, tokens),
    }
}

/// Parse a prefix operator expression, or fall through to a primary expression.
/// After the expression, trailing postfix operators are consumed inline
/// (before juxtaposition in the main loop).
#[allow(clippy::result_large_err)]
fn parse_prefix_or_primary(tokens: &mut TokenIter) -> Result<ParsedExpr> {
    // Prefix: try to read the longest matching prefix operator
    let mut expr = match try_read_longest_op(tokens, |op| prefix_bp(op).is_some()) {
        Some(op) => {
            // Prefix RHS allows juxtaposition: -f x = -(f x)
            let rhs = parse_expr(tokens, JUXTAPOSITION_BP)?;
            ParsedExpr::Prefix {
                op,
                expr: Box::new(rhs),
            }
        }
        None => parse_primary(tokens)?,
    };

    // Postfix: consume trailing postfix operators inline
    while let Some(op) = try_read_longest_op(tokens, |op| postfix_bp(op).is_some()) {
        expr = ParsedExpr::Postfix {
            expr: Box::new(expr),
            op,
        };
    }

    Ok(expr)
}

/// Main expression parser using precedence climbing.
#[allow(clippy::result_large_err)]
fn parse_expr(tokens: &mut TokenIter, min_bp: u32) -> Result<ParsedExpr> {
    let mut lhs = parse_prefix_or_primary(tokens)?;

    loop {
        // Juxtaposition: two adjacent primary expressions
        if peek_is_primary_start(tokens) {
            if JUXTAPOSITION_BP < min_bp {
                break;
            }
            let rhs = parse_expr(tokens, JUXTAPOSITION_BP + 1)?;
            lhs = match lhs {
                ParsedExpr::Juxtaposition(mut v) if !matches!(rhs, ParsedExpr::Postfix { .. }) => {
                    v.push(rhs);
                    ParsedExpr::Juxtaposition(v)
                }
                _ => ParsedExpr::Juxtaposition(vec![lhs, rhs]),
            };
            continue;
        }

        // Quick peek to see if the next token is a Punct at all
        if !matches!(tokens.clone().next(), Some(TokenTree::Punct(_))) {
            break;
        }

        // Try infix operator with transaction for backtracking
        let result = tokens.transaction(|t| -> Result<(String, ParsedExpr)> {
            let op = read_operator(t).ok_or_else(Error::no_error)?;

            if let Some((lbp, rbp)) = infix_bp(&op)
                && lbp >= min_bp
                && peek_is_expr_start(t)
            {
                let rhs = parse_expr(t, rbp)?;
                return Ok((op, rhs));
            }

            Err(Error::no_error())
        });

        match result {
            Ok((op, rhs)) => {
                lhs = ParsedExpr::Infix {
                    left: Box::new(lhs),
                    op,
                    right: Box::new(rhs),
                };
            }
            Err(_) => break,
        }
    }

    Ok(lhs)
}

impl Parser for ParsedExpr {
    fn parser(tokens: &mut TokenIter) -> Result<Self> {
        parse_expr(tokens, 0)
    }
}

struct CodeGen {
    counter: u32,
    defs: Vec<TokenStream>,
}

impl CodeGen {
    fn new() -> Self {
        Self {
            counter: 0,
            defs: Vec::new(),
        }
    }

    fn next_ident(&mut self) -> Ident {
        let id = self.counter;
        self.counter += 1;
        Ident::new(&format!("__N{}", id), proc_macro2::Span::call_site())
    }

    fn emit(&mut self, expr: &ParsedExpr) -> Ident {
        match expr {
            ParsedExpr::Atom(s) => {
                let name = self.next_ident();
                self.defs.push(quote! {
                    const #name: ::abs_expr::Expr<'static> = ::abs_expr::Expr::Atom(#s);
                });
                name
            }
            ParsedExpr::Juxtaposition(children) => {
                let child_names: Vec<_> = children.iter().map(|c| self.emit(c)).collect();
                let name = self.next_ident();
                self.defs.push(quote! {
                    const #name: ::abs_expr::Expr<'static> = ::abs_expr::Expr::Juxtaposition(
                        &[#(#child_names),*]
                    );
                });
                name
            }
            ParsedExpr::Grouped(inner) => self.emit(inner),
            ParsedExpr::Prefix { op, expr } => {
                let child = self.emit(expr);
                let name = self.next_ident();
                self.defs.push(quote! {
                    const #name: ::abs_expr::Expr<'static> = ::abs_expr::Expr::Prefix {
                        op: #op,
                        expr: &#child,
                    };
                });
                name
            }
            ParsedExpr::Postfix { expr, op } => {
                let child = self.emit(expr);
                let name = self.next_ident();
                self.defs.push(quote! {
                    const #name: ::abs_expr::Expr<'static> = ::abs_expr::Expr::Postfix {
                        expr: &#child,
                        op: #op,
                    };
                });
                name
            }
            ParsedExpr::Infix { left, op, right } => {
                let left_name = self.emit(left);
                let right_name = self.emit(right);
                let name = self.next_ident();
                self.defs.push(quote! {
                    const #name: ::abs_expr::Expr<'static> = ::abs_expr::Expr::Infix {
                        left: &#left_name,
                        op: #op,
                        right: &#right_name,
                    };
                });
                name
            }
        }
    }

    fn finalize(self, root: Ident) -> TokenStream {
        let defs = self.defs;
        quote! { const {
            #(#defs)*
            #root
        }}
    }
}

#[proc_macro]
pub fn abs_expr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let mut iter = TokenIter::new(input);

    match (&mut iter).parse_all::<ParsedExpr>() {
        Ok(expr) => {
            let mut cg = CodeGen::new();
            let root = cg.emit(&expr);
            cg.finalize(root).into()
        }
        Err(e) => {
            panic!("Parse error: {}", e);
        }
    }
}
