use ive_macros::node;
use quote::{quote, ToTokens};

#[derive(Debug)]
struct Node {
    name: String,
    inputs: Vec<TypeDef>,
    outputs: Vec<TypeDef>,
    template_args: Vec<String>,
}

#[derive(Debug)]
struct TypeDef {
    name: String,
    ty: String,
    template_args: Vec<String>,
}


#[node]
fn simple_function(x: u32) -> u32 {
    x + 1
}

#[node]
fn simple_function_str(instr: &str) -> String {
    format!("{}{}", instr, instr)
}

#[node]
fn template_function_type_to_string<T>(input: &T) -> String where T: std::fmt::Display {
    format!("{}", input)
}

#[test]
fn test_token_stream_to_node() {
    let res = simple_function(1);
    assert_eq!(res, 2);
    let node = _get_ive_node_info_simple_function();
    assert_eq!(node.name, "simple_function");
    assert_eq!(node.outputs.len(), 1);
    assert_eq!(node.outputs[0].name, "output");
    assert_eq!(node.outputs[0].ty, "u32");

    assert_eq!(node.inputs.len(), 1);
    assert_eq!(node.inputs[0].name, "x");
    assert_eq!(node.inputs[0].ty, "u32");
    assert_eq!(node.inputs[0].template_args.len(), 0, "{:?}", node.inputs[0].template_args);

    assert_eq!(node.template_args.len(), 0);
    //        assert_eq!(node.name, "simple_function");
}

#[test]
fn test_simple_function_str() {
    let res = simple_function_str("hello");
    assert_eq!(res, "hellohello");
    let node = _get_ive_node_info_simple_function_str();
    assert_eq!(node.name, "simple_function_str");
    assert_eq!(node.outputs.len(), 1);
    assert_eq!(node.outputs[0].name, "output");
    assert_eq!(node.outputs[0].ty, "String");
    assert_eq!(node.outputs[0].template_args.len(), 0);

    assert_eq!(node.inputs.len(), 1);
    assert_eq!(node.inputs[0].name, "instr");
    assert_eq!(node.inputs[0].ty, "& str");
    assert_eq!(node.inputs[0].template_args.len(), 0);
}

#[test]
fn test_template() {
    let res = template_function_type_to_string(&1);
    assert_eq!(res, "1");
    let node = _get_ive_node_info_template_function_type_to_string();
    assert_eq!(node.name, "template_function_type_to_string");
    assert_eq!(node.outputs.len(), 1);
    assert_eq!(node.outputs[0].name, "output");
    assert_eq!(node.outputs[0].ty, "String");
    assert_eq!(node.outputs[0].template_args.len(), 0);

    assert_eq!(node.inputs.len(), 1);
    assert_eq!(node.inputs[0].name, "input");
    assert_eq!(node.inputs[0].ty, "& T");
    assert_eq!(node.inputs[0].template_args.len(), 0);

    assert_eq!(node.template_args.len(), 1);
    assert_eq!(node.template_args[0], "T");

    println!("{:#?}", node);
}

#[derive(Debug, Clone, Copy)]
struct Dummy {
    x: u32,
}
fn take_ownership(x: Dummy) -> u32 {
    x.x
}

#[test]
fn test_copyable() {    
    let d = Dummy { x: 5 };
    let r = take_ownership(d);
    assert_eq!(r, 5);
    //let v = take_ownership(d);
}

#[test]
fn test_expression_quote() {
    let ast = quote!(run(foo,bar,blech,[], [one]));
    assert!(!ast.to_string().is_empty());

    
}

#[test]
fn test_quote() {
    let ast = quote!(
        fn template_function_type_to_string<T>(input: &T) -> String where T: std::fmt::Display {
            format!("{}", input)
        }
    );
    assert!(!ast.to_string().is_empty());

    let input_fn = syn::parse_str::<syn::ItemFn>(ast.to_string().as_str()).unwrap();
    let fnname = input_fn.sig.ident.to_string();
    assert_eq!(fnname, "template_function_type_to_string");

    let template_parameters = input_fn.sig.generics.params.iter().map(|x| {
        match x {
            syn::GenericParam::Type(ty) => {
                ty.ident.to_string()
            },
            _ => panic!("not a type"),
        }
    }).collect::<Vec<_>>();

    assert_eq!(template_parameters.len(), 1, "{:?}", template_parameters);
    assert_eq!(template_parameters[0], "T", "{:?}", template_parameters);

    let first_arg = input_fn.sig.inputs.iter().next().unwrap();
    let first_arg_name = match first_arg {
        syn::FnArg::Typed(ty) => {
            match ty.pat.as_ref() {
                syn::Pat::Ident(ident) => {
                    ident.ident.to_string()
                },
                _ => panic!("not an ident"),
            }
        },
        _ => panic!("not a typed"),
    };
    assert_eq!(first_arg_name, "input");

    let first_arg_type = match first_arg {
        syn::FnArg::Typed(ty) => {
            let ty = &*ty.ty;
            match ty {
                syn::Type::Reference(ty) => {
                    let ty = &*ty.elem;
                    match ty {
                        syn::Type::Path(ty) => {
                            let ty = &ty.path;
                            // let ty = &ty.segments;
                            // let ty = &ty[0];
                            // let ty = &ty.ident;
                            format!("& {}",ty.to_token_stream())
                        },
                        _ => panic!("not a path"),
                    }
                },
                _ => ty.to_token_stream().to_string(),
            }

        },
        _ => panic!("not a typed"),
    };
    assert_eq!(first_arg_type, "& T");
}