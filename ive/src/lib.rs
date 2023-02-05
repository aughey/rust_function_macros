use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Type};

fn type_to_string(kind: &Type) -> String {
    kind.to_token_stream().to_string()
}

#[proc_macro_attribute]
pub fn node(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let name = input_fn.sig.ident.to_string();
    let return_type = &input_fn.sig.output;
    let return_type = match return_type {
        syn::ReturnType::Default => "void".to_string(),
        syn::ReturnType::Type(_, ty) => type_to_string(ty),
    };

    let template_args = input_fn
        .sig
        .generics
        .type_params()
        .map(|ty| ty.ident.to_string())
        .map(|ty| quote! { #ty.to_string() })
        .collect::<Vec<_>>();

    let info_name = format_ident!("_get_ive_node_info_{}", name);
    let outputname = "output";

    let inputs = input_fn.sig.inputs.iter().map(|arg| {
        let name = arg
            .to_token_stream()
            .to_string()
            .split(":")
            .next()
            .unwrap()
            .trim()
            .to_string();
        let ty = match arg {
            syn::FnArg::Typed(ty) => type_to_string(&ty.ty),
            _ => "void".to_string(),
        };
       
        quote! {
            TypeDef {
                name: #name.to_string(),
                ty: #ty.to_string(),
                template_args: vec![
                ],
            }
        }
    }).collect::<Vec<_>>();

    quote!(
       #input_fn

       fn #info_name() -> Node {
            Node {
                name: #name.to_string(),
                inputs: vec![
                    #(#inputs),*
                ],
                outputs: vec![
                    TypeDef {
                        name: #outputname.to_string(),
                        ty: #return_type.to_string(),
                        template_args: vec![],
                    }
                ],
                template_args: vec![
                    #(#template_args),*
                ],
            }
       }
    )
    .into()
}
