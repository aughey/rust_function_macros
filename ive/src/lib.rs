use convert_case::{Casing, Case};
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Type};

fn type_to_string(kind: &Type) -> String {
    kind.to_token_stream().to_string()
}

// fn temp_vars(count: usize) -> Vec<Ident> {
//     (0..count)
//         .into_iter()
//         .map(|x| format_ident!("var{}", x))
//         .collect()
// }

// fn create_operation(
//     runstate: String,
//     output: String,
//     inputs: &[String],
//     function: String,
//     children: &[String],
// ) -> syn::Macro {
//     let myinputs = temp_vars(inputs.len());
//     let inputs = inputs
//         .iter()
//         .map(|x| parse_quote!(#x))
//         .collect::<Vec<syn::Expr>>();

//     let inputout = if inputs.len() > 0 {
//         quote! {
//             myinputs: (#(#myinputs),*) = (#(#inputs),*),
//         }
//     } else {
//         quote! {}
//     };

//     let children = children
//         .iter()
//         .map(|x| parse_quote!(#x))
//         .collect::<Vec<syn::Expr>>();

//     let runop: syn::Macro = parse_quote! {
//         run_operation!(
//             runstate: #runstate,
//             output: #output,
//             #inputout
//             function: #function,
//             children: #(#children),*
//         );
//     };

//     runop
// }

fn to_camel(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

type PMIdent = proc_macro2::Ident;
fn call<'a, I>(
    inputs: I,
    index: usize,
    function: &PMIdent,
    children: impl Iterator<Item = &'a usize>,
) -> proc_macro2::TokenStream
where
    I: Iterator<Item = PMIdent>,
    I: Clone,
{
    let some_temps = inputs.clone().map(|x| quote!(Some(#x))).collect::<Vec<_>>();
    let state_inputs = inputs.clone().map(|x| quote!(state.#x));
    let num_inputs = some_temps.len();
    let pull_inputs = match num_inputs {
        0 => quote! {},
        1 => quote! { if let #(#some_temps)* = #(#state_inputs)* },
        _ => quote! { let (#(#some_temps),*) = (#(#state_inputs),*); },
    };

    let output = format_ident!("value{}", index);

    let getdirtystate = quote!(dirty.state[#index]);
    let setdirtyclean = quote!(dirty.state[#index] = DirtyEnum::Clean);
    let setdirtystale = quote!(dirty.state[#index] = DirtyEnum::Stale);

    let children = children.map(|x| quote!(dirty.state[#x] = DirtyEnum::NeedCompute;));

    let else_inputs_invalid = match num_inputs {
        0 => quote! {},
        _ => quote! { else { #setdirtystale; None } },
    };

    quote!(
        if #getdirtystate == DirtyEnum::NeedCompute {
            state.#output = #pull_inputs {
                #setdirtyclean;
                compute_count += 1;
                Some(#function(#(#inputs),*))
            } #else_inputs_invalid;
            #(#children)*
        }
    )
}


#[proc_macro]
pub fn ive_chain(input: TokenStream) -> TokenStream {
    let count = parse_macro_input!(input as syn::LitInt);
    let count = count.base10_parse::<usize>().unwrap();

    let countrange = 0..count;

    let netname = "chain";
    let netname_camel = to_camel(netname);
    let fnname = format_ident!("{}", netname);

    let statename = format_ident!("{}State", netname_camel);
    let dirtyname = format_ident!("{}Dirty", netname_camel);

    let state_types = countrange
        .clone()
        .map(|_| format_ident!("u32"))
        .map(|ty| quote!(Option<#ty>));

    let state_names = countrange.clone().map(|i| format_ident!("value{}", i));

    let state_struct = {
        let state_values =
            std::iter::zip(state_types, state_names).map(|(kind, name)| quote!(#name : #kind));
        quote!(
            #[derive(Default,Copy,Clone)]
            pub struct #statename {
                #(pub #state_values),*
            }
        )
    };

    // let dirty_names = countrange.clone().map(|i| format_ident!("run{}", i));
    // let dirty_values = dirty_names.clone().map(|name| quote!(#name : DirtyEnum));

    let dirty_struct = {
        quote!(
            #[derive(Copy,Clone)]
            pub struct #dirtyname {
                //#(#dirty_values),*
                pub state: [DirtyEnum; #count]
            }
            impl Default for #dirtyname {
                fn default() -> Self {
                    Self {
                        state: [DirtyEnum::NeedCompute; #count]
                    }
                }
            }
            impl #dirtyname {
                #[inline(always)]
                pub fn get(&self, index: usize) -> DirtyEnum {
                    self.state[index]
                }
                pub fn set_needs_compute(&mut self, index: usize) {
                    self.state[index] = DirtyEnum::NeedCompute;
                }
            }

            // impl #dirtyname {
            //     #[inline(always)]
            //     pub fn dirty_state(&self, index: usize) -> DirtyEnum {
            //         self.state[index]
            //     }
            //     #[inline(always)]
            //     pub fn set_state(&mut self, index: usize, value: DirtyEnum) {
            //         self.state[index] = value;
            //     }
            // }
        )
    };

    let operations = countrange
        .clone()
        .skip(1) // Skip the first one
        .map(|i| {
            let input_indices = &[i - 1];
            let inputs = input_indices.iter().map(|x| format_ident!("value{}", x));

            let children: Vec<usize> = if i == count - 1 {
                Vec::<usize>::new()
            } else {
                vec![i + 1usize]
            };
            let function = format_ident!("add_one");

            call(inputs, i, &function, children.iter())
        });

    let firstcall = call(
        Vec::<PMIdent>::new().into_iter(),
        0,
        &format_ident!("zero"),
        vec![1].iter(),
    );

    let operations = operations.collect::<Vec<_>>();

    const CHUNKSIZE: usize = 200;
    let chunks = operations.chunks(CHUNKSIZE);

    fn make_chunk_name(fnname: &PMIdent, i: usize) -> PMIdent {
        format_ident!("{}_chunk{}", fnname, i)
    }

    let chunk_funcs = chunks.clone().enumerate().map(|(i, chunk)| {
        let chunkname = make_chunk_name(&fnname, i);
        //let chunk = chunk.iter().map(|x| quote!(#x));
        quote!(
            #[inline(never)]
            fn #chunkname(state: &mut #statename, dirty: &mut #dirtyname) -> usize {
                let mut compute_count : usize = 0;
                #(#chunk)*
                compute_count
            }
        )
    });
    let call_chunks = chunks.enumerate().map(|(i, _)| {
        let chunkname = make_chunk_name(&fnname, i);
        quote!(compute_count += #chunkname(state, dirty);)
    });

    let straightline_fn = {
        let sl_name = format_ident!("{}_straightline", fnname);
        let operations = countrange
        .skip(1) // Skip the first one
        .map(|i| {
            let input = format_ident!("value{}", i - 1);
            let output = format_ident!("value{}", i);
            let function = format_ident!("add_one");
            quote!(state.#output = Some(#function(state.#input.unwrap()));)
        });
        quote!(
            #[inline(never)]
            pub fn #sl_name(state: &mut #statename, dirty: &mut #dirtyname) -> usize {
                //let mut compute_count : usize = 0;
                state.value0 = Some(zero());
                #(#operations)*
                #count // sort of cheating
            }
        )
    };

    let out = quote! {
      #state_struct
      #dirty_struct

        #[inline(never)]
      pub fn #fnname(state: &mut #statename, dirty: &mut #dirtyname) -> usize {
        let mut compute_count : usize = 0;
        #firstcall
        #(#call_chunks)*
        compute_count
      }
      #(#chunk_funcs)*
      #straightline_fn
    };

//    eprintln!("{}", out);

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
                .split(':')
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

// struct JohnAugheyDyn;
// impl DynCall for JohnAugheyDyn {
//     fn call(&self, _inputs: &[&BoxedAny]) -> BoxedAny {
//         BoxedAny::new(john_aughey())
//     }
//     fn input_len(&self) -> usize {
//         0
//     }
// }

#[proc_macro_attribute]
pub fn make_dynamicable(_metadata: TokenStream, stream: TokenStream) -> TokenStream {
    let input : ItemFn = syn::parse(stream).unwrap();

    let ItemFn { attrs, vis, sig, block } = input;

    // Pull apart sig
    let fnname = &sig.ident;
    let inputs = &sig.inputs;
    let input_len = inputs.len();

    let dyncall_name = format_ident!("{}DynCall", fnname.to_string().to_case(Case::Pascal));

    enum DecomposableType {
        Option,
        Result
    }
    struct InputType {
        ty: proc_macro2::TokenStream,
        is_ref: bool
    }

    let input_types = inputs.iter().map(|arg| {
        match arg {
            syn::FnArg::Typed(ty) => {
               match &*ty.ty {
                     syn::Type::Reference(ty) => {
                        if ty.mutability.is_some() {
                            panic!("Cannot have a mutable reference as an input");
                        }
                        let ty = &*ty.elem;
                        InputType{ ty: quote!{#ty}, is_ref: true }
                     }
                     syn::Type::Path(ty) => {
                        let ty = &ty.path;
                        let first = ty.segments.first().unwrap().ident.to_string();
                        if ty.segments.len() == 1 && COPYABLE_TYPES.iter().any(|v| *v==first.as_str()) {
                            InputType{ ty: quote!{#ty}, is_ref: false }
                        } else {
                            panic!("Cannot have a non-copyable type as an input");
                        }
                     }
                     _ => panic!("Expected typed argument"),
               }
            },
            _ => panic!("Expected typed argument"),
        }
    });

    let input_pull = input_types.enumerate().map(|(i, ty)| {
        let kind = ty.ty;
        if ty.is_ref {
            quote! {
            //    inputs[#i].value::<#kind>()
                inputs.fetch::<#kind>(#i)?
            }
        } else {
            quote! {
                //*inputs[#i].value::<#kind>()
                *inputs.fetch::<#kind>(#i)?
            }
        }
    });

    // Look at the output and see if it's a special type we might be able to decompose.
    // (Optional or Result)
    let output_type = if let syn::ReturnType::Type(_, ty) = &sig.output {
        match &**ty {
            syn::Type::Path(ty) => {
                let ty = &ty.path;
                let first = ty.segments.first().unwrap().ident.to_string();
                match (ty.segments.len(), first.as_str()) {
                    (1, "Option") => Some(DecomposableType::Option),
                    (1, "Result") => Some(DecomposableType::Result),
                    _ => None
                }
            }
            _ => None
        }
    } else {
        None
    };

    let output_len = match output_type {
        Some(DecomposableType::Option) => 2,
        Some(DecomposableType::Result) => 2,
        None => 1
    };
    let output_len = quote!{#output_len as usize};

    let output_store  = match output_type {
        Some(DecomposableType::Option) => {
            quote! {
                if let Some(output) = output {
                    outputs.some(0,output);
                    outputs.none(1);
                } else {
                    outputs.none(0);
                    outputs.some(1,true);
                }
            }
        },
        Some(DecomposableType::Result) => {
            quote! {
                match output {
                    Ok(output) => {
                        outputs.some(0,output);
                        outputs.none(1);
                    },
                    Err(e) => {
                        outputs.none(0);
                        outputs.some(1,e);
                    }
                }
            }
        },
        None => {
            quote! {
                outputs.some(0,output);
            }
        }
    };

    let wrapper = quote!{
        struct #dyncall_name;
        impl DynCall for #dyncall_name {
            fn call(&self, inputs: &InputGetter, outputs: &mut OutputSetter) -> DynCallResult {
                assert_eq!(inputs.len(), #input_len, "Expected {} inputs, got {}", #input_len, inputs.len());
                assert_eq!(outputs.len(), #output_len, "Expected {} outputs, got {}", #output_len, outputs.len());
                let output = #fnname(#(#input_pull),*);
                #output_store
                Ok(())
            }
            fn input_len(&self) -> usize {
                #input_len
            }
            fn output_len(&self) -> usize {
                #output_len
            }
        }
    };

    quote! {
        #(#attrs)* #vis #sig #block
        #wrapper
    }.into()
}

const COPYABLE_TYPES : &[&str] = &[
    "std::cmp::Ordering",
    "Infallible",
    "std::fmt::Alignment",
    "ErrorKind",
    "SeekFrom",
    "IpAddr",
    "Ipv6MulticastScope",
    "Shutdown",
    "SocketAddr",
    "FpCategory",
    "BacktraceStyle",
    "Which",
    "SearchStep",
    "std::sync::atomic::Ordering",
    "RecvTimeoutError",
    "TryRecvError",
    "bool",
    "char",
    "f32",
    "f64",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "isize",
    "!",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "()",
    "usize",
    "CpuidResult",
    "__m128",
    "__m128bh",
    "__m128d",
    "__m128i",
    "__m256",
    "__m256bh",
    "__m256d",
    "__m256i",
    "__m512",
    "__m512bh",
    "__m512d",
    "__m512i",
    "AllocError",
    "Global",
    "Layout",
    "System",
    "TypeId",
    "TryFromSliceError",
    "CharTryFromError",
    "TryFromCharError",
    "Error",
    "FileTimes",
    "FileType",
    "Empty",
    "Sink",
    "Assume",
    "Ipv4Addr",
    "Ipv6Addr",
    "SocketAddrV4",
    "SocketAddrV6",
    "NonZeroI8",
    "NonZeroI16",
    "NonZeroI32",
    "NonZeroI64",
    "NonZeroI128",
    "NonZeroIsize",
    "NonZeroU8",
    "NonZeroU16",
    "NonZeroU32",
    "NonZeroU64",
    "NonZeroU128",
    "NonZeroUsize",
    "TryFromIntError",
    "RangeFull",
    "UCred",
    "ExitCode",
    "ExitStatus",
    "ExitStatusError",
    "std::ptr::Alignment",
    "Utf8Error",
    "RecvError",
    "WaitTimeoutResult",
    "RawWakerVTable",
    "AccessError",
    "ThreadId",
    "Duration",
    "Instant",
    "SystemTime",
    "PhantomPinned",
    "Level",
    "Span",
    "LineColumn",
    "Delimiter",
    "Spacing",
    "Metric",
    "ShouldPanic",
    "ColorConfig",
    "OutputFormat",
    "RunIgnored",
    "RunStrategy",
    "Options",
    "Summary",
    "TestTimeOptions",
    "TestType",
    "NamePadding",
    "TestId",
];