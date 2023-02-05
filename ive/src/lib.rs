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

    let info_name = format_ident!("_get_ive_node_info_{}", name);
    let outputname = format_ident!("output");

    quote!(
       #input_fn

       fn #info_name() -> Node {
            Node {
                name:"foo".to_string(),
                inputs: vec![

                ],
                outputs: vec![
                    // TypeDef {
                    //     name: #outputname,
                    //     ty: #return_type,
                    //     template_args: vec![],
                    // }
                ],
                template_args: vec![],
            }
       }
    )
    .into()
}
