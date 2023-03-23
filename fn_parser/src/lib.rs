use anyhow::Result;
use quote::ToTokens;

pub mod file_parsing;

pub struct TypeDefinition {
    pub tokens: Vec<String>,
}

pub struct InputDefinition {
    pub name: String,
    pub ty: TypeDefinition,
}

pub struct FunctionDefinition {
    pub name: String,
    pub inputs: Vec<InputDefinition>,
    pub output: TypeDefinition,
}

fn type_to_type_definition(ty: &syn::Type) -> Result<TypeDefinition> {
    let tokens_as_string = ty
        .to_token_stream()
        .into_iter()
        .map(|ts| ts.to_string())
        .collect::<Vec<String>>();
    Ok(TypeDefinition {
        tokens: tokens_as_string,
    })
}

fn return_to_type_definition(ret: &syn::ReturnType) -> Result<TypeDefinition> {
    match ret {
        syn::ReturnType::Type(_, ty) => type_to_type_definition(ty),
        _ => Ok(TypeDefinition { tokens: vec![] }),
    }
}

pub fn function_to_function_definition(f: &syn::ItemFn) -> Result<FunctionDefinition> {
    let inputs = f
        .sig
        .inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Receiver(_) => panic!("Receiver not supported"),
            syn::FnArg::Typed(typed) => {
                let name = typed.pat.to_token_stream().to_string();
                let ty = type_to_type_definition(&typed.ty)?;
                Ok(InputDefinition { name, ty })
            }
        })
        .collect::<Result<Vec<InputDefinition>>>()?;

    let output = return_to_type_definition(&f.sig.output)?;

    Ok(FunctionDefinition {
        name: f.sig.ident.to_string(),
        inputs,
        output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        let test_fn = r#"
            fn test_fn(a: u32, b: u64) -> u128 {
                0
            }
        "#;
        let itemfn = syn::parse_str::<syn::ItemFn>(test_fn).unwrap();
        let function_definition = function_to_function_definition(&itemfn).unwrap();
        assert_eq!(function_definition.name, "test_fn");
        assert_eq!(function_definition.inputs.len(), 2);
        assert_eq!(function_definition.inputs[0].name, "a");
        assert_eq!(function_definition.inputs[0].ty.tokens, vec!["u32"]);
        assert_eq!(function_definition.inputs[1].name, "b");
        assert_eq!(function_definition.inputs[1].ty.tokens, vec!["u64"]);
        assert_eq!(function_definition.output.tokens, vec!["u128"]);
    }

    #[test]
    fn test_reference_arguments() {
        let test_fn = r#"
            fn test_fn(a: &u32, b: &mut u64) -> u128 {
                0
            }
        "#;
        let itemfn = syn::parse_str::<syn::ItemFn>(test_fn).unwrap();
        let function_definition = function_to_function_definition(&itemfn).unwrap();
        assert_eq!(function_definition.name, "test_fn");
        assert_eq!(function_definition.inputs.len(), 2);
        assert_eq!(function_definition.inputs[0].name, "a");
        assert_eq!(function_definition.inputs[0].ty.tokens, vec!["&", "u32"]);
        assert_eq!(function_definition.inputs[1].name, "b");
        assert_eq!(
            function_definition.inputs[1].ty.tokens,
            vec!["&", "mut", "u64"]
        );
        assert_eq!(function_definition.output.tokens, vec!["u128"]);
    }

    #[test]
    fn test_reference_outputs() {
        let test_fn = r#"
            fn test_fn(a: u32, b: u64) -> &u128 {
                0
            }
        "#;
        let itemfn = syn::parse_str::<syn::ItemFn>(test_fn).unwrap();
        let function_definition = function_to_function_definition(&itemfn).unwrap();
        assert_eq!(function_definition.name, "test_fn");
        assert_eq!(function_definition.inputs.len(), 2);
        assert_eq!(function_definition.inputs[0].name, "a");
        assert_eq!(function_definition.inputs[0].ty.tokens, vec!["u32"]);
        assert_eq!(function_definition.inputs[1].name, "b");
        assert_eq!(function_definition.inputs[1].ty.tokens, vec!["u64"]);
        assert_eq!(function_definition.output.tokens, vec!["&", "u128"]);
    }
}
