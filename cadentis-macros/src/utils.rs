use proc_macro::{TokenStream, TokenTree};

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

fn is_arrow(tokens: &[TokenTree], i: usize) -> bool {
    if i + 1 >= tokens.len() {
        return false;
    }

    matches!((&tokens[i], &tokens[i + 1]),
        (TokenTree::Punct(p1), TokenTree::Punct(p2))
        if p1.as_char() == '=' && p2.as_char() == '>'
    )
}

pub(crate) fn parse_select_branches(input: TokenStream) -> Vec<(String, String)> {
    let args = split_args(input);
    let mut branches = Vec::new();

    for arg in args {
        let tokens = arg;
        let mut i = 0;

        let mut future_tokens = Vec::new();
        while i < tokens.len() {
            if is_arrow(&tokens, i) {
                i += 2;
                break;
            }
            future_tokens.push(tokens[i].clone());
            i += 1;
        }

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
