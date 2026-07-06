/*
Expr syntax:
Atom: an identifier or a literal
Juxtaposition: a sequence of expressions, e.g. `a b (c+d)`
Prefix: an operator followed by an expression, e.g. `-a`
Postfix: an expression followed by an operator, e.g. `a!`
Infix: an expression followed by an operator and another expression, e.g. `a + b`
Expressions can be nested with parentheses, e.g. `(a + b) * c`.
An operator can be any sequence of symbols, e.g. `+`, `++`, `->`, `::`, etc.
The precedence and associativity of operators are determined by the first character of the operator,
e.g. `+` and `-` have the same precedence and are left-associative, while `*` and `/` have higher precedence and are also left-associative.
The precedence and associativity rules are the same as in OCaml.
Refer to the OCaml documentation for more details: https://ocaml.org/manual/expr.html
*/

use proc_macro2::{Delimiter, Ident, Spacing, TokenStream, TokenTree};
use quote::quote;
use unsynn::*;

/// Heap-based parse tree produced by the parser.
/// Distict from `abs_expr::Expr<'a>` (the public reference-based type).
#[derive(Debug)]
// #[allow(dead_code)]
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
const POSTFIX_BP: u32 = 190;
const JUXTAPOSITION_BP: u32 = 185;
const MULTIPLICATIVE_BP: u32 = 170; // * / %
const ADDITIVE_BP: u32 = 160; // + -
const CONS_BP: u32 = 150; // :
const CONCAT_BP: u32 = 140; // @ ^
const COMPARISON_BP: u32 = 130; // = < > | & $ #
const COMMA_BP: u32 = 100; // ,
const SEMICOLON_BP: u32 = 80; // ;

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
        "!" | "?" | "~" | "-" | "-." => Some(PREFIX_BP),
        _ => None,
    }
}

/// Get binding power for a postfix operator.
fn postfix_bp(op: &str) -> Option<u32> {
    match op {
        "!" => Some(POSTFIX_BP),
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
#[allow(clippy::result_large_err)]
fn parse_prefix_or_primary(tokens: &mut TokenIter) -> Result<ParsedExpr> {
    // Quick peek: only proceed if the first char is a prefix-operator character
    {
        let mut clone = tokens.clone();
        match clone.next() {
            Some(TokenTree::Punct(p)) if matches!(p.as_char(), '!' | '?' | '~' | '-') => {}
            _ => return parse_primary(tokens),
        }
    }

    let result = tokens.transaction(|t| -> Result<String> {
        let op = read_operator(t).ok_or_else(Error::no_error)?;
        if prefix_bp(&op).is_some() {
            Ok(op)
        } else {
            Err(Error::no_error())
        }
    });

    match result {
        Ok(op) => {
            let expr = parse_expr(tokens, PREFIX_BP - 1)?;
            Ok(ParsedExpr::Prefix {
                op,
                expr: Box::new(expr),
            })
        }
        Err(_) => parse_primary(tokens),
    }
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
                ParsedExpr::Juxtaposition(mut v) => {
                    v.push(rhs);
                    ParsedExpr::Juxtaposition(v)
                }
                _ => ParsedExpr::Juxtaposition(vec![lhs, rhs]),
            };
            continue;
        }

        // Quick peek to see if the next token is a Punct at all
        let mut clone = tokens.clone();
        if !matches!(clone.next(), Some(TokenTree::Punct(_))) {
            break;
        }

        // Try operator (infix or postfix) with transaction for backtracking
        let result = tokens.transaction(|t| -> Result<(bool, String, ParsedExpr)> {
            let op = read_operator(t).ok_or_else(Error::no_error)?;

            // Try infix
            if let Some((lbp, rbp)) = infix_bp(&op)
                && lbp >= min_bp
                && peek_is_expr_start(t)
            {
                let rhs = parse_expr(t, rbp)?;
                return Ok((true, op, rhs));
            }

            // Try postfix
            if let Some(pbp) = postfix_bp(&op)
                && pbp >= min_bp
            {
                return Ok((false, op, ParsedExpr::Atom(String::new())));
            }

            Err(Error::no_error())
        });

        match result {
            Ok((true, op, rhs)) => {
                lhs = ParsedExpr::Infix {
                    left: Box::new(lhs),
                    op,
                    right: Box::new(rhs),
                };
            }
            Ok((false, op, _)) => {
                lhs = ParsedExpr::Postfix {
                    expr: Box::new(lhs),
                    op,
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

// ---------------------------------------------------------------------------
// Code generation: walk the ParsedExpr tree and emit const items
// ---------------------------------------------------------------------------

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
        quote! {{
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
