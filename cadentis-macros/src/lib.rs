mod utils;

use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn join(input: TokenStream) -> TokenStream {
    let args = utils::split_args(input);
    let count = args.len();

    if count == 0 {
        return "()".parse().unwrap();
    }

    if count == 1 {
        let expr = args[0]
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join("");
        return format!("{{ {}.await }}", expr).parse().unwrap();
    }

    let mut output = String::new();
    output.push_str("{\n");

    for (i, expr_tokens) in args.iter().enumerate() {
        let idx = i + 1;
        let expr = utils::tokens_to_string(expr_tokens);
        output.push_str(&format!(
            "let mut __f{idx} = (::std::boxed::Box::pin({expr}), ::core::option::Option::None::<_>, false);\n"
        ));
    }

    output.push_str("::std::future::poll_fn(move |cx| {\n");
    output.push_str("    use ::std::task::Poll;\n");

    for i in 1..=count {
        output.push_str(&format!(
            "    if !__f{i}.2 {{\n\
                    if let Poll::Ready(val) = __f{i}.0.as_mut().poll(cx) {{\n\
                        __f{i}.1 = ::core::option::Option::Some(val);\n\
                        __f{i}.2 = true;\n\
                    }}\n\
                }}\n"
        ));
    }

    let all_done = (1..=count)
        .map(|i| format!("__f{i}.2"))
        .collect::<Vec<_>>()
        .join(" && ");

    output.push_str(&format!("    if {all_done} {{\n"));
    output.push_str("        Poll::Ready((\n");

    for i in 1..=count {
        output.push_str(&format!("            __f{i}.1.take().unwrap(),\n"));
    }

    output.push_str("        ))\n");
    output.push_str("    } else {\n");
    output.push_str("        Poll::Pending\n");
    output.push_str("    }\n");
    output.push_str("}).await\n");
    output.push_str("}\n");

    match output.parse::<TokenStream>() {
        Ok(ts) => ts,
        Err(err) => {
            let msg = format!("join_impl macro error: {}", err);
            format!("compile_error!(\"{}\");", msg).parse().unwrap()
        }
    }
}

#[proc_macro]
pub fn select(input: TokenStream) -> TokenStream {
    let branches = utils::parse_select_branches(input);
    let count = branches.len();

    if count == 0 {
        return "()".parse().unwrap();
    }

    let mut out = String::new();
    out.push_str("{\n");

    let generics = (1..=count)
        .map(|i| format!("__T{i}"))
        .collect::<Vec<_>>()
        .join(", ");

    out.push_str(&format!("enum __SelectResult<{generics}> {{\n"));
    for i in 1..=count {
        out.push_str(&format!("    __F{i}(__T{i}),\n"));
    }
    out.push_str("}\n\n");

    for (i, (future, _handler)) in branches.iter().enumerate() {
        let idx = i + 1;
        out.push_str(&format!(
            "let mut __f{idx} = ::std::boxed::Box::pin({future});\n"
        ));
    }

    out.push_str("\nlet __res = ::std::future::poll_fn(move |cx| {\n");
    out.push_str("    use ::std::task::Poll;\n");
    out.push_str("    use ::std::future::Future;\n");

    for i in 1..=count {
        out.push_str(&format!(
            "    if let Poll::Ready(val) = __f{i}.as_mut().poll(cx) {{\n\
                 return Poll::Ready(__SelectResult::__F{i}(val));\n\
             }}\n"
        ));
    }

    out.push_str("    Poll::Pending\n");
    out.push_str("}).await;\n\n");

    out.push_str("let __out = match __res {\n");
    for (i, (_future, handler)) in branches.iter().enumerate() {
        let idx = i + 1;
        out.push_str(&format!(
            "    __SelectResult::__F{idx}(val) => {{ ({handler})(val) }},\n"
        ));
    }
    out.push_str("};\n");
    out.push_str("__out\n");
    out.push_str("}\n");

    out.parse().unwrap_or_else(|err| {
        let msg = format!("select macro error: {err}");
        format!("compile_error!(\"{}\");", msg).parse().unwrap()
    })
}

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut tokens: Vec<TokenTree> = item.into_iter().collect();

    let attr_str = attr.to_string();
    let mut worker_threads: Option<usize> = None;

    if !attr_str.is_empty() {
        for part in attr_str.split(',') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("worker_threads") {
                let v = v.trim_start_matches('=').trim();
                worker_threads = v.parse::<usize>().ok();
            }
        }
    }

    let Some(pos) = tokens.iter().rposition(
        |t| matches!(t, TokenTree::Group(g) if g.delimiter() == proc_macro::Delimiter::Brace),
    ) else {
        return TokenStream::new();
    };

    let block = match &tokens[pos] {
        TokenTree::Group(g) => g.stream().to_string(),
        _ => unreachable!(),
    };

    let mut builder = String::from("::cadentis::RuntimeBuilder::new()");

    if let Some(n) = worker_threads {
        builder.push_str(&format!(".worker_threads({})", n));
    }

    builder.push_str(".build()");

    if let Some(async_pos) = tokens
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(id) if id.to_string() == "async"))
    {
        tokens.remove(async_pos);
    }

    let new_block = format!(
        "{{
            let runtime = {};
            runtime
                .block_on(async move {{
                    {}
                }})
        }}",
        builder, block
    );

    tokens[pos] = TokenTree::Group(proc_macro::Group::new(
        proc_macro::Delimiter::Brace,
        new_block.parse().unwrap(),
    ));

    tokens.into_iter().collect()
}

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut tokens = item.into_iter().collect::<Vec<_>>();

    if let Some(pos) = tokens
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(id) if id.to_string() == "async"))
    {
        tokens.remove(pos);
    }

    let block_pos = tokens.iter().rposition(
        |t| matches!(t, TokenTree::Group(g) if g.delimiter() == proc_macro::Delimiter::Brace),
    );

    let Some(pos) = block_pos else {
        return TokenStream::new();
    };

    let block = match &tokens[pos] {
        TokenTree::Group(g) => g.stream().to_string(),
        _ => unreachable!(),
    };

    let new_block = format!(
        "{{
        let runtime = ::cadentis::RuntimeBuilder::new().build();
        runtime
            .block_on(async move {{ {} }});
    }}",
        block
    );

    tokens[pos] = TokenTree::Group(proc_macro::Group::new(
        proc_macro::Delimiter::Brace,
        new_block.parse().unwrap(),
    ));

    let test_attr: TokenStream = "#[test]".parse().unwrap();
    let mut result: Vec<TokenTree> = test_attr.into_iter().collect();
    result.extend(tokens);

    result.into_iter().collect()
}
