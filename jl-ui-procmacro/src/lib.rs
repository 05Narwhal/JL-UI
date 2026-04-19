/*
Copyright (C) 2026 Julian Loy

This file is part of jl_ui.

jl_ui is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License v3.
*/

use std::iter::Peekable;

use proc_macro::{Ident, TokenStream, TokenTree};

fn next_group(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<proc_macro::Group> {
    if let Some(TokenTree::Group(_)) = source.peek() {
        let group = match source.next().unwrap() {
            TokenTree::Group(group) => group,
            _ => unreachable!("just checked with peek()!"),
        };
        Some(group)
    } else {
        None
    }
}

fn next_literal(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<String> {
    if let Some(TokenTree::Literal(lit)) = source.peek() {
        let mut literal = lit.to_string();

        // the only way to check that literal is string :/
        if literal.starts_with("\"") {
            literal.remove(0);
            literal.remove(literal.len() - 1);
        }
        source.next();
        return Some(literal);
    }

    return None;
}

#[proc_macro_attribute]
pub fn jlui_init(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut modified = TokenStream::new();
    let mut source = item.into_iter().peekable();

    let mut crate_rename = None;

    while let Some(TokenTree::Punct(punct)) = source.peek() {
        assert_eq!(format!("{}", punct), "#");

        // skip '#'
        let _ = source.next().unwrap();

        let group = next_group(&mut source);
        let mut group = group.unwrap().stream().into_iter().peekable();
        let attribute_name = format!("{}", group.next().unwrap());

        // skipping non-relevant attributes
        if attribute_name == "macroquad" {
            let group = next_group(&mut group);
            let mut group = group.unwrap().stream().into_iter().peekable();
            let config_name = format!("{}", group.next().unwrap());

            if group.peek().is_some() {
                // skip '='
                let _ = group.next();

                let config_value = Some(next_literal(&mut group).unwrap());

                if config_name == "crate_rename" {
                    crate_rename = config_value;
                }
            }
        }
    }

    let is_pub = if let TokenTree::Ident(ident) = source.peek().unwrap() {
        if ident.to_string() == "pub" {
            // skip 'pub'
            let _ = source.next().unwrap();
            true
        } else {
            false
        }
    } else {
        false
    };

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "async");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[jlui_init] is allowed only for async functions");
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "fn");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[jlui_init] is allowed only for functions");
    }

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "main");

        modified.extend(std::iter::once(TokenTree::Ident(Ident::new(
            "amain",
            ident.span(),
        ))));
    } else {
        panic!("[jlui_init] expecting main function");
    }

    modified.extend(std::iter::once(source.next().unwrap()));

    let next = source.next().unwrap();
    let use_result = if let TokenTree::Punct(punct) = &next {
        format!("{}", punct) == "-" // Start of `-> Result<(), ...>`
    } else {
        false
    };
    modified.extend(std::iter::once(next));
    modified.extend(source);

    let config_expr = match attr.into_iter().next() {
        Some(TokenTree::Ident(ident)) => format!("{}().to_macroquad_config()", ident),
        Some(TokenTree::Literal(literal)) => format!("({}).to_macroquad_config()", literal),
        Some(wrong_ident) => panic!(
            "Wrong argument: {:?}. Place function returned `WindowConfig`",
            wrong_ident
        ),
        None => panic!("No argument! Place function returned `WindowConfig`"),
    };

    let crate_name = crate_rename.unwrap_or_else(|| "macroquad".to_string());
    let mut prelude: TokenStream = format!(
        "
    {pub_main} fn main() {{
        {crate_name}::Window::from_config({config_expr}, {main});
    }}
    ",
        pub_main = if is_pub { "pub" } else { "" },
        crate_name = crate_name,
        config_expr = config_expr,
        main = if use_result {
            format!(
                "async {{
                if let Err(err) = amain().await {{
                    {}::logging::error!(\"Error: {{:?}}\", err);
                }}
            }}",
                crate_name
            )
        } else {
            "amain()".to_string()
        }
    )
    .parse()
    .unwrap();
    prelude.extend(modified);

    prelude
}