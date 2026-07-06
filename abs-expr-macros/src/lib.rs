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

use proc_macro2::{Delimiter, Spacing, TokenTree};
use unsynn::*;

#[derive(Debug)]
#[allow(dead_code)]
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

// Operator precedence levels (higher = tighter binding)
const PREFIX_BP: u32 = 210;
const EXPONENTIATION_BP: u32 = 195; // ** right
const POSTFIX_BP: u32 = 190;
const JUXTAPOSITION_BP: u32 = 185;
const MULTIPLICATIVE_BP: u32 = 170; // * / % - left
const ADDITIVE_BP: u32 = 160; // + - - left
const CONS_BP: u32 = 150; // :: - right
const CONCAT_BP: u32 = 140; // @ ^ - right
const COMPARISON_BP: u32 = 130; // = < > | & $ # - left
const CONJUNCTION_BP: u32 = 120; // & && - right
const DISJUNCTION_BP: u32 = 110; // || - right
const COMMA_BP: u32 = 100; // , - left
const ASSIGNMENT_BP: u32 = 90; // <- := - right
const SEMICOLON_BP: u32 = 80; // ; - right

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
    // Full-operator special cases that differ from the first-character rule
    match op {
        "**" | "**." => return Some((EXPONENTIATION_BP, EXPONENTIATION_BP)),
        "<-" | ":=" => return Some((ASSIGNMENT_BP, ASSIGNMENT_BP)),
        "||" => return Some((DISJUNCTION_BP, DISJUNCTION_BP)),
        "&&" => return Some((CONJUNCTION_BP, CONJUNCTION_BP)),
        ";" | ";;" => return Some((SEMICOLON_BP, SEMICOLON_BP)),
        _ => {}
    }

    let first = op.chars().next()?;
    match first {
        '|' => Some((COMPARISON_BP, COMPARISON_BP + 1)),
        '&' => Some((COMPARISON_BP, COMPARISON_BP + 1)),
        '=' | '$' | '#' => Some((COMPARISON_BP, COMPARISON_BP + 1)),
        '<' | '>' => Some((COMPARISON_BP, COMPARISON_BP + 1)),
        ':' => Some((CONS_BP, CONS_BP)),
        '@' | '^' => Some((CONCAT_BP, CONCAT_BP)),
        '+' => Some((ADDITIVE_BP, ADDITIVE_BP + 1)),
        '-' => Some((ADDITIVE_BP, ADDITIVE_BP + 1)),
        '*' | '/' | '%' => Some((MULTIPLICATIVE_BP, MULTIPLICATIVE_BP + 1)),
        ',' => Some((COMMA_BP, COMMA_BP + 1)),
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
fn parse_primary(tokens: &mut TokenIter) -> Result<Expr> {
    match tokens.next() {
        Some(TokenTree::Ident(ident)) => Ok(Expr::Atom(ident.to_string())),
        Some(TokenTree::Literal(lit)) => Ok(Expr::Atom(lit.to_string())),
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            let mut inner = TokenIter::new(group.stream());
            parse_expr(&mut inner, 0)
        }
        _ => Error::unexpected_token(None, tokens),
    }
}

/// Parse a prefix operator expression, or fall through to a primary expression.
#[allow(clippy::result_large_err)]
fn parse_prefix_or_primary(tokens: &mut TokenIter) -> Result<Expr> {
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
            Ok(Expr::Prefix {
                op,
                expr: Box::new(expr),
            })
        }
        Err(_) => parse_primary(tokens),
    }
}

/// Main expression parser using precedence climbing.
#[allow(clippy::result_large_err)]
fn parse_expr(tokens: &mut TokenIter, min_bp: u32) -> Result<Expr> {
    let mut lhs = parse_prefix_or_primary(tokens)?;

    loop {
        // Juxtaposition: two adjacent primary expressions
        if peek_is_primary_start(tokens) {
            if JUXTAPOSITION_BP < min_bp {
                break;
            }
            let rhs = parse_expr(tokens, JUXTAPOSITION_BP + 1)?;
            lhs = match lhs {
                Expr::Juxtaposition(mut v) => {
                    v.push(rhs);
                    Expr::Juxtaposition(v)
                }
                _ => Expr::Juxtaposition(vec![lhs, rhs]),
            };
            continue;
        }

        // Quick peek to see if the next token is a Punct at all
        {
            let mut clone = tokens.clone();
            if !matches!(clone.next(), Some(TokenTree::Punct(_))) {
                break;
            }
        }

        // Try operator (infix or postfix) with transaction for backtracking
        let result = tokens.transaction(|t| -> Result<(bool, String, Expr)> {
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
            if let Some(pbp) = postfix_bp(&op) && pbp >= min_bp {
                return Ok((false, op, Expr::Atom(String::new())));
            }

            Err(Error::no_error())
        });

        match result {
            Ok((true, op, rhs)) => {
                lhs = Expr::Infix {
                    left: Box::new(lhs),
                    op,
                    right: Box::new(rhs),
                };
            }
            Ok((false, op, _)) => {
                lhs = Expr::Postfix {
                    expr: Box::new(lhs),
                    op,
                };
            }
            Err(_) => break,
        }
    }

    Ok(lhs)
}

impl Parser for Expr {
    fn parser(tokens: &mut TokenIter) -> Result<Self> {
        parse_expr(tokens, 0)
    }
}

#[proc_macro]
pub fn abs_expr(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let mut iter = TokenIter::new(input);

    match (&mut iter).parse_all::<Expr>() {
        Ok(expr) => {
            let repr = format!("{:?}", expr);
            let lit = proc_macro2::Literal::string(&repr);
            let tt = proc_macro2::TokenTree::Literal(lit);
            let ts = proc_macro2::TokenStream::from(tt);
            proc_macro::TokenStream::from(ts)
        }
        Err(e) => {
            panic!("Parse error: {}", e);
        }
    }
}
