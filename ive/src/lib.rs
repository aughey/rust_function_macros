use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Ident, ItemFn, Type, parse_quote};

fn type_to_string(kind: &Type) -> String {
    kind.to_token_stream().to_string()
}

fn temp_vars(count: usize) -> Vec<Ident> {
    (0..count)
        .into_iter()
        .map(|x| format_ident!("var{}", x))
        .collect()
}

fn create_operation(
    runstate: String,
    output: String,
    inputs: &[String],
    function: String,
    children: &[String],
) -> syn::Macro {
    let myinputs = temp_vars(inputs.len());
    let inputs = inputs
        .iter()
        .map(|x| parse_quote!(#x))
        .collect::<Vec<syn::Expr>>();

    let inputout = if inputs.len() > 0 {
        quote! {
            myinputs: (#(#myinputs),*) = (#(#inputs),*),
        }
    } else {
        quote! {}
    };

    let children = children
        .iter()
        .map(|x| parse_quote!(#x))
        .collect::<Vec<syn::Expr>>();

    let runop : syn::Macro = parse_quote! {
        run_operation!(
            runstate: #runstate,
            output: #output,
            #inputout
            function: #function,
            children: #(#children),*
        );
    };

    runop
}

#[proc_macro]
pub fn ive_chain(input: TokenStream) -> TokenStream {
    let count = parse_macro_input!(input as syn::LitInt);
    let count = count.base10_parse::<usize>().unwrap();

    let operations = (1..count)
        .into_iter()
        .map(|i| {
            create_operation(
                format!("runstate.op{}", i).to_string(),
                format!("state.op{}", i).to_string(),
                &[format!("state.op{}", i - 1).to_string()],
                "add1".to_string(),
                &[format!("runstate.op{}", i + 1).to_string()],
            )
        })
        .collect::<Vec<_>>();


    let out = quote! {
        fn chain_test() {
    //        #ops
        }
    };

    out.into()
}

#[proc_macro]
pub fn run_node(input: TokenStream) -> TokenStream {
    let _args = parse_macro_input!(input as syn::Expr);

    quote!({}).into()
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

    let inputs = input_fn
        .sig
        .inputs
        .iter()
        .map(|arg| {
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
        })
        .collect::<Vec<_>>();

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
