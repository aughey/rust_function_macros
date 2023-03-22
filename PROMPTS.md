I need help writing rust code to extract information from a function that was parsed using syn.

First I have the following structures:

struct TypeDefinition {
    // All of the tokens used in the type declaration
    type_tokens: Vec<String>
}
struct FunctionArgument {
    // Name of the argument.
    name: String,
    // All of the tokens used in the type declaration
    type_tokens: TypeDefinition
}
struct FunctionDefinition {
    fn_name: String,
    inputs: Vec<FunctionArgument>,
    output: TypeDefinition
}

I want you to implement the function
fn parse_function(f: f: ItemFn) -> FunctionDefinition