use crate::{function_to_function_definition, FunctionDefinition};
use anyhow::Result;

pub fn parse_file(path: &str) -> Result<impl IntoIterator<Item = FunctionDefinition>> {
    let file = std::fs::read_to_string(path)?;
    parse_file_str(&file)
}

pub fn parse_file_str(content: &str) -> Result<impl IntoIterator<Item = FunctionDefinition>> {
    let file_content = syn::parse_file(content)?;

    let functions : Vec<_> = file_content
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(f) => Some(f),
            _ => None,
        })
        .filter_map(|f| function_to_function_definition(f).ok())
        .collect();
    Ok(functions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        let file_content = r#"
            fn foo() {}
            fn bar() {}
        "#;
        let functions = parse_file_str(file_content).unwrap();
        assert_eq!(functions.into_iter().count(), 2);
    }
}