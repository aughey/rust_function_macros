use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, ItemFn, Type};

pub type TokenResult<O> = Result<O, syn::Error>;

struct PathWrapper<'a> {
    path: &'a syn::Path,
}
struct ReferenceTypeWrapper<'a> {
    rt: &'a syn::TypeReference,
}
struct TypeWrapper<'a> {
    ty: &'a syn::Type,
}
impl TypeWrapper<'_> {
    fn type_strings(&self) -> impl Iterator<Item = String> {
        self.ty.to_token_stream().into_iter().map(|x| x.to_string())
    }
}
struct PatTypeWrapper<'a> {
    ty: &'a syn::PatType,
}
impl<'a> PatTypeWrapper<'a> {
    fn is_ref(&self) -> bool {
        match *self.ty.ty {
            syn::Type::Reference(_) => true,
            _ => false,
        }
    }
    fn tokens(&self) -> proc_macro2::TokenStream {
        self.ty.ty.to_token_stream()
    }
    fn name(&self) -> TokenResult<syn::Ident> {
        let pat = &*self.ty.pat;
        match pat {
            syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
            _ => Err(syn::Error::new(self.ty.pat.span(), "Expected identifier")),
        }
    }

    fn type_strings(&self) -> impl Iterator<Item = String> {
        self.tokens().into_iter().map(|x| x.to_string())
    }
}
struct FnArgWrapper<'a> {
    arg: &'a syn::FnArg,
}
impl<'a> FnArgWrapper<'a> {
    fn typed(&self) -> TokenResult<PatTypeWrapper> {
        match self.arg {
            syn::FnArg::Typed(ty) => Ok(PatTypeWrapper { ty: ty }),
            _ => Err(syn::Error::new(self.arg.span(), "Expected typed argument")),
        }
    }

    fn is_ref(&self) -> TokenResult<bool> {
        Ok(self.typed()?.is_ref())
    }

    fn tokens(&self) -> TokenResult<proc_macro2::TokenStream> {
        Ok(self.typed()?.tokens())
    }

    fn name(&self) -> TokenResult<syn::Ident> {
        self.typed()?.name()
    }
}
struct FunctionWrapper<'a> {
    input_fn: &'a ItemFn,
}

impl<'a> FunctionWrapper<'a> {
    fn name(&self) -> &'a syn::Ident {
        &self.input_fn.sig.ident
    }
    fn inputs(&self) -> impl Iterator<Item = FnArgWrapper<'a>> + Clone {
        self.input_fn
            .sig
            .inputs
            .iter()
            .map(|arg| FnArgWrapper { arg })
    }
    fn output(&self) -> Option<TypeWrapper<'a>> {
        match &self.input_fn.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(TypeWrapper { ty: ty }),
        }
    }
}

pub fn make_dynamicable_work(f: ItemFn) -> TokenResult<TokenStream> {
    let fw = FunctionWrapper { input_fn: &f };

    // let ItemFn {
    //     attrs,
    //     vis,
    //     sig,
    //     block,
    // } = f;

    // Pull apart sig
    //    let fnname = fw.name_ident();
    // let inputs = sig.inputs;
    // let input_len = inputs.len();

    // let dyncall_name = format_ident!("{}DynCall", fnname.to_string().to_case(Case::Pascal));

    // enum DecomposableType {
    //     Option,
    //     Result,
    // }
    // struct InputType {
    //     ty: proc_macro2::TokenStream,
    //     name: Box<syn::Pat>,
    //     is_ref: bool,
    // }

    //  let inputs = fw.inputs();

    // let zzzinput_types = f.sig.inputs.iter().map(|arg| match arg {
    //     syn::FnArg::Typed(pat_ty) => match &*pat_ty.ty {
    //         syn::Type::Reference(ty) => {
    //             if ty.mutability.is_some() {
    //                 panic!("Cannot have a mutable reference as an input");
    //             }
    //             let ty = &*ty.elem;
    //             InputType {
    //                 ty: quote! {#ty},
    //                 name: pat_ty.pat.clone(),
    //                 is_ref: true,
    //             }
    //         }
    //         syn::Type::Path(ty) => {
    //             let ty = &ty.path;
    //             let first = ty.segments.first().unwrap().ident.to_string();
    //             if ty.segments.len() == 1 && COPYABLE_TYPES.iter().any(|v| *v == first.as_str()) {
    //                 InputType {
    //                     ty: quote! {#ty},
    //                     name: pat_ty.pat.clone(),
    //                     is_ref: false,
    //                 }
    //             } else {
    //                 panic!("Cannot have a non-copyable type as an input");
    //             }
    //         }
    //         _ => panic!("Expected typed argument"),
    //     },
    //     _ => panic!("Expected typed argument"),
    // });

    // // Create the quote code to pull out the inputs
    // let input_pull = inputs.clone().enumerate().map(|(i, ty)| {
    //     let deref = if ty.is_ref()? {
    //         quote! {}
    //     } else {
    //         quote! { * }
    //     };
    //     let ty = ty.tokens()?;
    //     Ok(quote!{
    //         #deref inputs[#i].value::<#ty>()?
    //     })
    // });

    // let input_pull = input_types.clone().enumerate().map(|(i, ty)| {
    //     let kind = ty.ty;
    //     if ty.is_ref {
    //         quote! {
    //         //    inputs[#i].value::<#kind>()
    //             inputs.fetch::<#kind>(#i)?
    //         }
    //     } else {
    //         quote! {
    //             //*inputs[#i].value::<#kind>()
    //             *inputs.fetch::<#kind>(#i)?
    //         }
    //     }
    // });

    // Look at the output and see if it's a special type we might be able to decompose.
    // (Optional or Result)
    //   let output = fw.output();
    // let output_type = if let syn::ReturnType::Type(_, ty) = &sig.output {
    //     match &**ty {
    //         syn::Type::Path(ty) => {
    //             let ty = &ty.path;
    //             let first = ty.segments.first().unwrap().ident.to_string();
    //             match (ty.segments.len(), first.as_str()) {
    //                 (1, "Option") => Some(DecomposableType::Option),
    //                 (1, "Result") => Some(DecomposableType::Result),
    //                 _ => None,
    //             }
    //         }
    //         _ => None,
    //     }
    // } else {
    //     None
    // };

    // let input_info = input_types.map(|ty| {
    //     let name = ty.name.to_token_stream().to_string();
    //     let kind = ty.ty.to_token_stream().into_iter().map(|t| t.to_string());
    //     quote! {
    //         ive::dyn_call::DynPort {
    //             name: #name,
    //             kind: vec![#(#kind.to_string()),*]
    //         }
    //     }
    // });

    // let output_info = match &sig.output {
    //     syn::ReturnType::Type(_, ty) => ty
    //         .to_token_stream()
    //         .into_iter()
    //         .map(|t| t.to_string())
    //         .collect::<Vec<_>>(),
    //     _ => vec![],
    // };

    // let output_len = match output_type {
    //     Some(DecomposableType::Option) => 2,
    //     Some(DecomposableType::Result) => 2,
    //     None => 1,
    // };
    // let output_len = quote! {#output_len as usize};

    // let output_store = match output_type {
    //     Some(DecomposableType::Option) => {
    //         quote! {
    //             if let Some(output) = output {
    //                 outputs.some(0,output);
    //                 outputs.none(1);
    //             } else {
    //                 outputs.none(0);
    //                 outputs.some(1,true);
    //             }
    //         }
    //     }
    //     Some(DecomposableType::Result) => {
    //         quote! {
    //             match output {
    //                 Ok(output) => {
    //                     outputs.some(0,output);
    //                     outputs.none(1);
    //                 },
    //                 Err(e) => {
    //                     outputs.none(0);
    //                     outputs.some(1,e);
    //                 }
    //             }
    //         }
    //     }
    //     None => {
    //         quote! {
    //             outputs.some(0,output);
    //         }
    //     }
    // };

    // let wrapper = quote! {
    //     pub struct #dyncall_name;
    //     impl ive::dyn_call::DynCall for #dyncall_name {
    //         fn call(&self, inputs: &ive::dyn_call::InputGetter, outputs: &mut ive::dyn_call::OutputSetter) -> ive::dyn_call::DynCallResult {
    //             assert_eq!(inputs.len(), #input_len, "Expected {} inputs, got {}", #input_len, inputs.len());
    //             assert_eq!(outputs.len(), #output_len, "Expected {} outputs, got {}", #output_len, outputs.len());
    //             let output = #fnname(#(#input_pull),*);
    //             #output_store
    //             Ok(())
    //         }
    //         fn input_len(&self) -> usize {
    //             #input_len
    //         }
    //         fn output_len(&self) -> usize {
    //             #output_len
    //         }
    //         fn inputs(&self) -> Vec::<ive::dyn_call::DynPort> {
    //             vec![ #(#input_info),*]
    //         }
    //         fn output_type(&self) -> &'static[&'static str] {
    //             &[ #(#output_info),* ]
    //         }
    //     }
    // };

    let wrapper = create_dyn_wrapper(&fw)?;

    Ok(quote! {
        #f
        #wrapper
    }
    .into())
}

fn pull_inputs<'a>(
    inputs: impl Iterator<Item = FnArgWrapper<'a>>,
) -> TokenResult<Vec<TokenStream>> {
    let pull = inputs.enumerate().map(|(i, ty)| {
        eprintln!("{}",ty.tokens().unwrap());
        let deref = if !ty.is_ref()? {
            quote! { * }
        } else {
            quote! {}
        };
        let ty = ty.tokens()?;
        Ok(quote! {
             #deref inputs.fetch::<#ty>(#i)?
        })
    });
    let tokens = pull.collect::<TokenResult<Vec<_>>>()?;
    Ok(tokens)
}

fn store_outputs(output: &TypeWrapper) -> TokenResult<TokenStream> {
    //    let output_type = output.ty.to_token_stream().into_iter().map(|t| t.to_string());
    Ok(quote! {
        outputs.some(0, output);
    })
}

fn input_len(fw: &FunctionWrapper) -> TokenStream {
    let len = fw.inputs().count();
    quote! {
        fn input_len(&self) -> usize {
            #len
        }
    }
}

fn output_len(_fw: &FunctionWrapper) -> TokenStream {
    quote! {
        fn output_len(&self) -> usize {
            1
        }
    }
}

fn call_dyncall(fw: &FunctionWrapper) -> TokenResult<TokenStream> {
    let fnname = fw.name();
    let input_pull = pull_inputs(fw.inputs())?;
    let output_store = if let Some(output) = fw.output() {
        store_outputs(&output)?
    } else {
        quote! {}
    };

    Ok(quote! {
        fn call(&self, inputs: &ive::dyn_call::InputGetter, outputs: &mut ive::dyn_call::OutputSetter) -> ive::dyn_call::DynCallResult {
            assert_eq!(inputs.len(), self.input_len(), "Expected {} inputs, got {}", self.input_len(), inputs.len());
            assert_eq!(outputs.len(), self.output_len(), "Expected {} outputs, got {}",self.output_len(), outputs.len());
            let output = #fnname(#(#input_pull),*);
            #output_store
            Ok(())
        }
    })
}

fn fn_arg_to_dynport(arg: &FnArgWrapper) -> TokenResult<TokenStream> {
    let ty = arg.typed()?;
    let name = arg.name()?.to_string();
    let kind = ty.type_strings(); // .tokens()?.into_iter().map(|t| t.to_string());
    Ok(quote! {
        ive::dyn_call::DynPort {
            name: #name,
            kind: vec![#(#kind.to_string()),*]
        }
    })
}

fn outputtype_dyncall(fw: &FunctionWrapper) -> TokenResult<TokenStream> {
    let output = fw.output();
    if let Some(output) = output {
        let output_type = output.type_strings();
        Ok(quote! {
            fn output_type(&self) -> &'static[&'static str] {
                &[ #(#output_type),* ]
            }
        })
    } else {
        Ok(quote! {
            fn output_type(&self) -> &'static[&'static str] {
                &[]
            }
        })
    }
}

fn inputs_dyncall(fw: &FunctionWrapper) -> TokenResult<TokenStream> {
    let input_info = fw.inputs().map(|i| fn_arg_to_dynport(&i));
    let input_info = input_info.collect::<TokenResult<Vec<_>>>()?;

    Ok(quote! {
        fn inputs(&self) -> Vec::<ive::dyn_call::DynPort> {
            vec![ #(#input_info),*]
        }
    })
}

fn impl_dyncall(fw: &FunctionWrapper) -> TokenResult<TokenStream> {
    let call = call_dyncall(fw)?;
    let il_fn = input_len(fw);
    let ol_fn = output_len(fw);
    let inputs = inputs_dyncall(fw)?;
    let output_type = outputtype_dyncall(fw)?;
    Ok(quote! {
        #call
        #il_fn
        #ol_fn
        #inputs
        #output_type
    })
}

fn create_dyn_wrapper(fw: &FunctionWrapper) -> TokenResult<TokenStream> {
    let dyncall_name = format_ident!("{}DynCall", fw.name().to_string().to_case(Case::Pascal));

    let impl_body = impl_dyncall(fw)?;

    Ok(quote! {
        pub struct #dyncall_name;
        impl ive::dyn_call::DynCall for #dyncall_name {
            #impl_body
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_make_dynamicable() {
        let testfn = quote!(
            pub fn testfn(a: i32, b: i32) -> i32 {
                a + b
            }
        );
        let parsed = syn::parse2::<syn::ItemFn>(testfn).unwrap();
        let output = make_dynamicable_work(parsed).unwrap();
        eprintln!("{}", output);
    }
}
