use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Meta, Pat};

use crate::{is_async_task, is_byte_slice, napi_function_meta, result_value_type, return_type};

pub(super) fn expand_function(function: &ItemFn, arguments: Vec<Meta>) -> syn::Result<TokenStream> {
    let async_task = is_async_task(&arguments);
    let mut native_arguments = Vec::new();
    let mut conversions = Vec::new();
    let mut call_arguments = Vec::new();
    let mut task_argument_names = Vec::new();
    let mut task_argument_types = Vec::new();
    for input in &function.sig.inputs {
        let FnArg::Typed(argument) = input else {
            return Err(syn::Error::new_spanned(
                input,
                "#[bridge] functions cannot have a receiver",
            ));
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &argument.pat,
                "#[bridge] parameters must use identifier patterns",
            ));
        };

        let ident = &pattern.ident;
        let semantic_type = &argument.ty;
        task_argument_names.push(ident);
        if is_byte_slice(semantic_type) {
            native_arguments.push(quote!(#ident: napi::bindgen_prelude::Buffer));
            call_arguments.push(quote!(&#ident));
            task_argument_types.push(quote!(napi::bindgen_prelude::Buffer));
            continue;
        }
        task_argument_types.push(quote!(#semantic_type));

        let decode_type = (**semantic_type).clone();
        native_arguments.push(quote! {
            #ident: <<#decode_type as ::swc_next_bridge::BridgeDecode>::Wire as
                ::swc_next_bridge::NativeAbi>::Abi
        });
        conversions.push(quote! {
            let #ident = <<#decode_type as ::swc_next_bridge::BridgeDecode>::Wire as
                ::swc_next_bridge::NativeAbi>::from_native(#ident);
            let Some(#ident) = <#decode_type as ::swc_next_bridge::BridgeDecode>::decode(#ident)
            else {
                return Err(napi::Error::new(
                    napi::Status::InvalidArg,
                    concat!("invalid bridge value for `", stringify!(#ident), "`"),
                ));
            };
        });
        call_arguments.push(quote!(#ident));
    }

    let declared_return_type = return_type(&function.sig.output);
    let (return_type, fallible) = result_value_type(&declared_return_type)
        .map_or((declared_return_type.clone(), false), |ty| {
            (ty.clone(), true)
        });

    let attributes = &function.attrs;
    let visibility = &function.vis;
    let unsafety = &function.sig.unsafety;
    let ident = &function.sig.ident;
    let native_adapter = format_ident!("__swc_bridge_napi_{ident}");
    let call = if unsafety.is_some() {
        quote!(unsafe { #ident(#(#call_arguments),*) })
    } else {
        quote!(#ident(#(#call_arguments),*))
    };
    let call = if fallible {
        quote! {
            let result = #call
                .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        }
    } else {
        quote!(let result = #call;)
    };
    let napi = napi_function_meta(arguments, function)?;
    let wire_type = quote!(<#return_type as ::swc_next_bridge::BridgeEncode>::Wire);
    let (task, native_return, body) = if async_task {
        (
            quote! {
                #visibility struct __SwcAsyncTask {
                    arguments: Option<(#(#task_argument_types,)*)>,
                }

                impl napi::Task for __SwcAsyncTask {
                    type Output = #wire_type;
                    type JsValue = <#wire_type as ::swc_next_bridge::NativeAbi>::Abi;

                    fn compute(&mut self) -> napi::Result<Self::Output> {
                        let (#(#task_argument_names,)*) = self.arguments
                            .take()
                            .expect("bridge task runs once");
                        #call
                        Ok(<#return_type as ::swc_next_bridge::BridgeEncode>::encode(result))
                    }

                    fn resolve(&mut self, _: napi::Env, output: Self::Output)
                        -> napi::Result<Self::JsValue>
                    {
                        Ok(<#wire_type as ::swc_next_bridge::NativeAbi>::into_native(output))
                    }
                }
            },
            quote!(napi::bindgen_prelude::AsyncTask<__SwcAsyncTask>),
            quote! {
                Ok(napi::bindgen_prelude::AsyncTask::new(__SwcAsyncTask {
                    arguments: Some((#(#task_argument_names,)*)),
                }))
            },
        )
    } else {
        (
            TokenStream::new(),
            quote!(<#wire_type as ::swc_next_bridge::NativeAbi>::Abi),
            quote! {
                #call
                let result = <#return_type as ::swc_next_bridge::BridgeEncode>::encode(result);
                Ok(<#wire_type as ::swc_next_bridge::NativeAbi>::into_native(result))
            },
        )
    };

    Ok(quote! {
        #[cfg(not(target_family = "wasm"))]
        const _: () = {
            // NAPI-RS emits relative `napi::` paths. Keep its import local to each
            // adapter so callers need neither a dependency nor a module-level import.
            use ::swc_next_bridge::napi;

            #task

            #(#attributes)*
            #[#napi]
            #visibility fn #native_adapter(
                #(#native_arguments),*
            ) -> napi::Result<#native_return> {
                #(#conversions)*
                #body
            }
        };
    })
}
