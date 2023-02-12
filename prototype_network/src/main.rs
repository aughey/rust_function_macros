use prototype_network::description::{Library, NodeDescriptor, PortDescriptor};

fn main() {
    let mut lib = Library::new("sample_nodes", "sample_nodes", "Sample nodes");
    lib.add_node(NodeDescriptor::new(
        "add_u32",
        "Add two u32 numbers together",
        vec![
            PortDescriptor::new("left", "Left input", "u32"),
            PortDescriptor::new("right", "Right input", "u32"),
        ],
        vec![PortDescriptor::new("output", "Output", "u32")],
    ));

    lib.add_node(NodeDescriptor::new(
        "three",
        "Return the number 3",
        vec![],
        vec![PortDescriptor::new("output", "Output", "u32")],
    ));

    lib.add_node(NodeDescriptor::new(
        "four",
        "Return the number 4",
        vec![],
        vec![PortDescriptor::new("output", "Output", "u32")],
    ));

    lib.add_node(NodeDescriptor::new(
        "copy_u32",
        "Copy a u32 value",
        vec![PortDescriptor::new("input", "Input", "u32")],
        vec![PortDescriptor::new("output", "Output", "u32")],
    ));

    let lib_json = serde_json::to_string_pretty(&lib).unwrap();
    println!("{}", lib_json);
}
