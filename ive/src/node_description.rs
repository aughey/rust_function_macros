pub struct NodeInfo {
    pub kind: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

pub struct Port {
    pub name : String,
    pub kind : Vec<String>
}
