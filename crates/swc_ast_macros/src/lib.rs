use proc_macro::TokenStream;
use syn::{Expr, Item, Meta, Token, parse_macro_input, punctuated::Punctuated};

mod ast;

#[proc_macro_attribute]
pub fn ast(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input = parse_macro_input!(input as Item);

    let mut repr = false;
    let mut nested = false;
    let mut storage = None;
    let mut id = None;
    for arg in args {
        match arg {
            Meta::Path(path) if path.is_ident("repr") => repr = true,
            Meta::Path(path) if path.is_ident("nested") => nested = true,
            Meta::NameValue(value) if value.path.is_ident("storage") => {
                let Expr::Path(path) = value.value else {
                    panic!("ast storage must be an identifier");
                };
                storage = path.path.get_ident().cloned();
            }
            Meta::NameValue(value) if value.path.is_ident("id") => {
                let Expr::Path(path) = value.value else {
                    panic!("ast id must be an identifier");
                };
                id = path.path.get_ident().cloned();
            }
            Meta::Path(path) => panic!("unknown ast option {}", quote::quote!(#path)),
            other => panic!("unknown ast option {}", quote::quote!(#other)),
        }
    }

    if storage.is_some() != id.is_some() {
        panic!("dedicated ast storage and typed id must be declared together");
    }
    if storage.as_ref().is_some_and(|storage| storage == "common") {
        panic!("common storage must use the default ast attribute");
    }

    TokenStream::from(ast::expand_ast(repr, nested, id, input))
}
