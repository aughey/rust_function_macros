use ive::node;

struct Node {
    name: String,
    inputs: Vec<TypeDef>,
    outputs: Vec<TypeDef>,
    template_args: Vec<String>,
}
struct TypeDef {
    name: String,
    ty: String,
    template_args: Vec<String>,
}


#[node]
fn simple_function(x: u32) -> u32 {
    x + 1
}

#[test]
fn test_token_stream_to_node() {
    let res = simple_function(1);
    assert_eq!(res, 2);
    let node = _get_ive_node_info_simple_function();
    assert_eq!(node.name, "simple_function");
    assert_eq!(node.outputs.len(), 1);
    //        assert_eq!(node.name, "simple_function");
}
