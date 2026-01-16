use proc_macro::{TokenStream, TokenTree};

/// Splits a `TokenStream` into comma-separated arguments.
///
/// Each argument is returned as a `Vec<TokenTree>`.
/// Commas at the top level are used as separators.
///
/// This function does **not** attempt to handle nested structures;
/// it assumes the input has already been tokenized appropriately
/// by the macro entry point.
pub(crate) fn split_args(input: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut args = Vec::new();
    let mut current = Vec::new();

    for token in input {
        match &token {
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if !current.is_empty() {
                    args.push(current);
                    current = Vec::new();
                }
            }
            _ => current.push(token),
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

/// Converts a slice of tokens into a Rust source string.
///
/// This function preserves token order and inserts spaces
/// between consecutive identifiers to avoid accidental
/// token merging (e.g. `foo bar` vs `foobar`).
pub(crate) fn tokens_to_string(tokens: &[TokenTree]) -> String {
    let mut out = String::new();
    let mut prev_was_ident = false;

    for t in tokens {
        let s = t.to_string();

        let needs_space = prev_was_ident && matches!(t, TokenTree::Ident(_));

        if needs_space {
            out.push(' ');
        }

        out.push_str(&s);
        prev_was_ident = matches!(t, TokenTree::Ident(_));
    }

    out
}

/// Returns `true` if the tokens at position `i` form a `=>` arrow.
///
/// This is used to detect branch boundaries in `select`-like
/// macro syntax.
fn is_arrow(tokens: &[TokenTree], i: usize) -> bool {
    if i + 1 >= tokens.len() {
        return false;
    }

    matches!(
        (&tokens[i], &tokens[i + 1]),
        (TokenTree::Punct(p1), TokenTree::Punct(p2))
            if p1.as_char() == '=' && p2.as_char() == '>'
    )
}

/// Parses `select`-style branches from a token stream.
///
/// Each branch is expected to have the form:
///
/// ```text
/// future_expr => handler_expr
/// ```
///
/// Multiple branches must be separated by commas.
///
/// The result is a list of `(future, handler)` pairs, both
/// returned as source strings.
///
/// Invalid or incomplete branches are ignored.
pub(crate) fn parse_select_branches(input: TokenStream) -> Vec<(String, String)> {
    let args = split_args(input);
    let mut branches = Vec::new();

    for arg in args {
        let tokens = arg;
        let mut i = 0;

        // Parse future expression (before `=>`)
        let mut future_tokens = Vec::new();
        while i < tokens.len() {
            if is_arrow(&tokens, i) {
                i += 2;
                break;
            }
            future_tokens.push(tokens[i].clone());
            i += 1;
        }

        // Parse handler expression (after `=>`)
        let mut handler_tokens = Vec::new();
        while i < tokens.len() {
            handler_tokens.push(tokens[i].clone());
            i += 1;
        }

        let future = tokens_to_string(&future_tokens);
        let handler = tokens_to_string(&handler_tokens);

        if !future.trim().is_empty() && !handler.trim().is_empty() {
            branches.push((future, handler));
        }
    }

    branches
}
