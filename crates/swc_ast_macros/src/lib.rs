use proc_macro::TokenStream;
use syn::{Ident, Item, Token, parse_macro_input, punctuated::Punctuated};

mod ast;

#[proc_macro_attribute]
pub fn ast(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Ident, Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as Item);
    let expanded = ast::expand_ast(
        args.iter().any(|arg| arg == "repr"),
        args.iter().any(|arg| arg == "nested"),
        input,
    );
    TokenStream::from(expanded)
}

/// Attach generator metadata without changing a scalar enum's Rust representation.
#[proc_macro_attribute]
pub fn estree_enum(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemEnum);
    for variant in &mut input.variants {
        variant.attrs.retain(|attr| !attr.path().is_ident("estree"));
    }
    quote::quote!(#input).into()
}
