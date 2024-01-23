#![feature(proc_macro_span)]

use std::{fs::File, io::Read};

use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use scraper::{Html, Selector};
use syn::{
    self,
    meta::ParseNestedMeta,
    parse::{self, Parse, ParseStream},
    parse_macro_input, LitStr,
};
use thiserror::Error;

#[derive(Debug)]
struct OmniTemplate {
    script: String,
    template: String,
}

#[derive(Debug)]
struct ComponentAttributes {
    tag: String,
}

// #[derive(Error, Debug)]
// enum ParseError {
//     #[error("unexpected attribute {0}")]
//     UnexpectedAttribute(String),
//     #[error("syn error")]
//     SynError(#[from] syn::Error),
// }

impl ComponentAttributes {
    fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("tag") {
            self.tag = meta.value()?.parse::<LitStr>()?.value();
            Ok(())
        } else {
            unimplemented!()
        }
    }
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the list of variables the user wanted to print.
    // let mut args = parse_macro_input!(attr as ComponentAttributes);
    let mut component_attributes = ComponentAttributes {
        tag: String::from(""),
    };
    let component_parser = syn::meta::parser(|meta| component_attributes.parse(meta));
    parse_macro_input!(attr with component_parser);

    println!("{:?}", item);
    item
}

#[proc_macro]
pub fn template_file(input: TokenStream) -> TokenStream {
    let span = Span::call_site();
    let source = span.source_file();

    let file_name = syn::parse::<syn::LitStr>(input).unwrap().value();
    // The parnet() eliminates the file name from the source path
    let file_path = source.path().parent().unwrap().join(file_name);
    println!("{:?}", file_path);
    let mut file = File::open(file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let component = Html::parse_fragment(&contents);
    let script = component
        .select(&Selector::parse("script").unwrap())
        .next()
        .unwrap();
    let template = component
        .select(&Selector::parse("template").unwrap())
        .next()
        .unwrap();

    // let file_tokens: TokenStream = contents.parse().unwrap();
    let _input = OmniTemplate {
        script: script.text().collect(),
        template: template.inner_html(),
    };
    println!("{:?}", _input);
    // 1. Use syn to parse the input tokens into a syntax tree.
    // 2. Use quote to generate new tokens based on what we parsed.
    // 3. Return the generated tokens.
    let html: proc_macro2::TokenStream =
        syn::LitStr::new(&_input.template, proc_macro2::Span::call_site())
            .token()
            .into_token_stream();
    quote! {
        return #html
    }
    .into()
}
