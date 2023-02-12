use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Library {
    name: String,
    crate_name: String,
    description: String,
    nodes: Vec<NodeDescriptor>,
}

impl Library {
    pub fn new(name: &str, crate_name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            crate_name: crate_name.to_string(),
            description: description.to_string(),
            nodes: vec![],
        }
    }

    pub fn add_node(&mut self, node: NodeDescriptor) {
        self.nodes.push(node);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeDescriptor {
    id: u64,
    name: String,
    description: String,
    inputs: Vec<PortDescriptor>,
    outputs: Vec<PortDescriptor>,
}

impl NodeDescriptor {
    pub fn new(
        name: &str,
        description: &str,
        inputs: Vec<PortDescriptor>,
        outputs: Vec<PortDescriptor>,
    ) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            description: description.to_string(),
            inputs,
            outputs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortDescriptor {
    id: String,
    name: String,
    description: String,
    ty: String,
}

impl PortDescriptor {
    pub fn new(name: &str, description: &str, ty: &str) -> Self {
        Self {
            id: name.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            ty: ty.to_string(),
        }
    }
}