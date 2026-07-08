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
const JUXTAPOSITION_BP: u32 = 185;
const MULTIPLICATIVE_BP: u32 = 170; // * / %
const ADDITIVE_BP: u32 = 160; // + -
const CONS_BP: u32 = 150; // :
const CONCAT_BP: u32 = 140; // @ ^
const COMPARISON_BP: u32 = 130; // = < > | & $ #
const COMMA_BP: u32 = 100; // ,
const SEMICOLON_BP: u32 = 80; // ;

/// Read Joint Puncts incrementally, advancing `tokens` past the longest
/// operator and returning it. On failure (no Punct), `tokens` may have
/// advanced past a non-Punct — the caller should snapshot before calling
/// and restore on None if needed.
fn read_op(tokens: &mut TokenIter) -> Option<String> {
    let mut op = String::new();
    let mut best: Option<(String, TokenIter)> = None;

    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(p) => {
                op.push(p.as_char());
                best = Some((op.clone(), tokens.clone()));
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
        None => None,
    }
}

/// Get left and right binding power for an infix operator.
///
/// Precedence is determined by the **last** character of the operator.
/// This makes `->` (last char `>`) a comparison-level operator rather than
/// additive-level, which is more intuitive. Single-character operators
/// are unaffected (first char == last char). Any unknown character
/// receives comparison-level precedence as a catch-all default.
fn infix_bp(op: &str) -> Option<(u32, u32)> {
    let c = op.chars().last()?;
    match c {
        '*' | '/' | '%' => Some((MULTIPLICATIVE_BP, MULTIPLICATIVE_BP + 1)),
        '+' | '-' => Some((ADDITIVE_BP, ADDITIVE_BP + 1)),
        ':' => Some((CONS_BP, CONS_BP)),
        '@' | '^' => Some((CONCAT_BP, CONCAT_BP)),
        '=' | '<' | '>' | '|' | '&' | '$' | '#' | '!' | '?' | '~' => {
            Some((COMPARISON_BP, COMPARISON_BP + 1))
        }
        ',' => Some((COMMA_BP, COMMA_BP + 1)),
        ';' => Some((SEMICOLON_BP, SEMICOLON_BP)),
        _ => Some((COMPARISON_BP, COMPARISON_BP + 1)),
    }
}

/// Check whether the next token is a primary expression start
/// (identifier, literal, or parenthesized group).
fn peek_is_primary(tokens: &TokenIter) -> bool {
    match tokens.clone().next() {
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

#[allow(clippy::result_large_err)]
fn parse_expr(tokens: &mut TokenIter, min_bp: u32) -> Result<ParsedExpr> {
    let mut lhs = match tokens.transaction(|t| -> Result<ParsedExpr> {
        let op = read_op(t).ok_or_else(Error::no_error)?;
        let rhs = parse_expr(t, JUXTAPOSITION_BP)?;
        Ok(ParsedExpr::Prefix {
            op,
            expr: Box::new(rhs),
        })
    }) {
        Ok(prefix) => prefix,
        Err(_) => parse_primary(tokens)?,
    };

    loop {
        // Juxtaposition: two adjacent primary expressions
        if peek_is_primary(tokens) {
            if JUXTAPOSITION_BP < min_bp {
                break;
            }
            let rhs = parse_expr(tokens, JUXTAPOSITION_BP + 1)?;
            lhs = match lhs {
                ParsedExpr::Juxtaposition(mut v)
                    if !matches!(rhs, ParsedExpr::Postfix { .. }) =>
                {
                    v.push(rhs);
                    ParsedExpr::Juxtaposition(v)
                }
                _ => ParsedExpr::Juxtaposition(vec![lhs, rhs]),
            };
            continue;
        }

        // Try to read an operator. Rolls back on no-Punct or
        // "leave for outer" (infix operator at too-low precedence).
        let Ok(op) = tokens.transaction(|t| -> Result<String> {
            let op = read_op(t).ok_or_else(Error::no_error)?;
            let bp = infix_bp(&op);
            // A primary follows but lbp is too low — an outer parse
            // level (with lower min_bp) might handle it.
            if let Some((lbp, _)) = bp && lbp < min_bp && peek_is_primary(t) {
                return Err(Error::no_error());
            }
            Ok(op)
        }) else {
            break;
        };

        // Try infix: sufficient precedence and a primary expression follows.
        let bp = infix_bp(&op);
        if let Some((lbp, rbp)) = bp
            && lbp >= min_bp
            && peek_is_primary(tokens)
        {
            match tokens.transaction(|t| parse_expr(t, rbp)) {
                Ok(rhs) => {
                    lhs = ParsedExpr::Infix {
                        left: Box::new(lhs),
                        op,
                        right: Box::new(rhs),
                    };
                    continue;
                }
                Err(_) => break,
            }
        }

        // Postfix
        lhs = ParsedExpr::Postfix {
            expr: Box::new(lhs),
            op,
        };
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
            let msg = format!("Parse error: {}", e);
            // Produce a compile_error!() token stream instead of panicking
            quote! { compile_error!(#msg) }.into()
        }
    }
}
