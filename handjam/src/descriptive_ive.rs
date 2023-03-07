use anyhow::anyhow;
use ive::dyn_call::{DynCall, DynLinearExec};
use std::cell::RefCell;
use std::rc::Rc;

type Id = String;
#[derive(Debug, PartialEq, Eq)]
pub struct Connection {
    pub from_id: Id,
    pub from_port: String,
    pub to_port: String,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub id: Id,
    pub kind: String,
    pub incoming_connections: Vec<Connection>,
    pub data: Option<String>,
}
#[derive(Default, Debug, PartialEq, Eq)]
pub struct PODGraph {
    pub nodes: Vec<Node>,
}
impl PODGraph {
    fn node_index(&self, id: &Id) -> anyhow::Result<usize> {
        self.nodes.iter().position(|n| n.id == *id).ok_or_else(|| anyhow!("Node {} not found",id))
    }
}

pub struct SortedGraph<'a> {
    sort: Vec<usize>,
    graph: &'a PODGraph,
}
impl<'a> IntoIterator for SortedGraph<'a> {
    type Item = &'a Node;
    type IntoIter = std::iter::Map<std::slice::Iter<'a, usize>, fn(&usize) -> &'a Node>;
    fn into_iter(self) -> Self::IntoIter {
        self.sort
            .iter()
            .map(|i| &self.graph.nodes[i])
    }   
}

pub fn pod_to_sorted(graph: &PODGraph) -> anyhow::Result<SortedGraph> {
    let mut g = petgraph::graph::Graph::<(), ()>::new();
    let handles = graph
        .nodes
        .iter()
        .map(|_| g.add_node(()))
        .collect::<Vec<_>>();
    for (i, node) in graph.nodes.iter().enumerate() {
        for (ci, connection) in node.incoming_connections.iter().enumerate() {
            let index = graph.node_index(&connection.from_id)?;
            let from = handles
                .get(index)
                .ok_or_else(|| anyhow!("Connection from index bad node {} connection {}", i, ci))?;
            let to = handles
                .get(i)
                .ok_or_else(|| anyhow!("bad handle index:  This one shouldn't ever happen"))?;
            g.add_edge(*from, *to, ());
        }
    }

    let topo = petgraph::algo::toposort(&g, None).map_err(|_| anyhow!("Cycle detected"))?;
    Ok(SortedGraph {
        sort: topo.into_iter().map(|t| t.index()).collect::<Vec<_>>(),
        graph,
    })
}

type BuilderGraph = Rc<RefCell<PODGraph>>;
pub struct PortBuilder {
    dag: BuilderGraph,
    index: usize,
    port: &'static str,
}
impl PortBuilder {
    pub fn connect_to(&mut self, other: &InPort) -> &mut Self {
        {
            let myid = self.dag.borrow().nodes[self.index].id.clone();
            let mut dag = self.dag.borrow_mut();
            let node = &mut dag.nodes[other.index];
            node.incoming_connections.push(Connection {
                from_id: myid,
                from_port: self.port.into(),
                to_port: other.port.into(),
            });
        }
        self
    }
}
pub struct InPort {
    index: usize,
    port: &'static str,
}

pub struct NodeBuilder {
    dag: BuilderGraph,
    index: usize,
}
impl NodeBuilder {
    pub fn in_port(&mut self, port: &'static str) -> InPort {
        InPort {
            index: self.index,
            port,
        }
    }
    pub fn out_port(&self, port: &'static str) -> PortBuilder {
        PortBuilder {
            dag: self.dag.clone(),
            index: self.index,
            port,
        }
    }
}

pub struct GraphBuilder {
    dag: BuilderGraph,
}
impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            dag: Rc::new(RefCell::new(PODGraph::default())),
        }
    }
    pub fn add_node(&mut self, kind: &'static str) -> NodeBuilder {
        let nodes = &mut self.dag.borrow_mut().nodes;
        let index = nodes.len();
        nodes.push(Node {
            id: uuid::Uuid::new_v4().to_string(),
            kind: kind.into(),
            incoming_connections: vec![],
            data: None,
        });
        NodeBuilder {
            dag: self.dag.clone(),
            index,
        }
    }
    pub fn build(self) -> PODGraph {
        self.dag.take()
    }
}

pub trait NodeFactory {
    fn create(&self, node: &Node) -> anyhow::Result<Box<dyn DynCall>>;
}



pub fn sorted_to_exec(sorted: &SortedGraph, factory: impl NodeFactory) -> anyhow::Result<DynLinearExec> {
    let compute_nodes = sorted
        .iter()
        .map(|node| factory.create(node))
        .collect::<anyhow::Result<Vec<_>>>()?;

    // Generate a list of the output indices
    let mut output_indices = vec![];
    { // populate output_indices
        output_indices.reserve_exact(compute_nodes.len());
        let mut accumm = 0;
        for node in compute_nodes.iter() {
            output_indices.push(accumm);
            accumm += node.output_len();
        }
    }
    
    let  exec = DynLinearExec::new(compute_nodes.into_iter());

//     // setup the connections
//     for ((i,node),computenode) in std::iter::zip(sorted.iter().enumerate(),compute_nodes.iter()) {
//         // for each real port:
//         let input_indices = computenode.inputs().iter().map(|p| {
//             // Look for an incoming connection
//             let connection = node
//                 .incoming_connections
//                 .iter()
//                 .find(|c| c.to_port == p.name)
//                 .ok_or_else(|| anyhow!("Missing connection for input {}", p.name))?;
//             // Find the output node
//             let from_position = sorted.node_position(&connection.from_id).ok_or_else(|| anyhow!("Could not find now for connection {}", p.name))?;
//             let from_port_index = 0; //compute_nodes[from_position].o XXX

//             Ok(output_indices[from_position] + from_port_index)
// //            let output_node = sorted.sorted_node(connection.from_index);
//         }).collect::<anyhow::Result<Vec<_>>>()?;
//         exec.inputs(i, input_indices);
//     }


    Ok(exec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ive::dyn_call::box_dyn_call;

    #[test]
    fn test_execution() {
        let nodes = vec![box_dyn_call(crate::OneDynCall {})];
        let mut exec = ive::dyn_call::DynLinearExec::new_linear_chain(nodes.into_iter());

        let count = exec.run().unwrap();
        assert_eq!(count, 1);
        let value1 = exec.value::<i32>(0).unwrap();
        assert_eq!(*value1, 1);
    }

    #[test]
    fn test_introspection() {
        let one = crate::OneDynCall {};

        let ot = one.output_type();
        assert_eq!(ot, &["i32"]);
        let inputs = one.inputs();
        assert_eq!(inputs.len(), 0);

        let add = crate::AddDynCall {};

        let ot = add.output_type();
        assert_eq!(ot, &["i32"]);
        let inputs = add.inputs();
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].name, "a".to_string());
        assert_eq!(inputs[0].kind, &["i32"]);
        assert_eq!(inputs[1].name, "b".to_string());
        assert_eq!(inputs[1].kind, &["i32"]);
    }

    #[test]
    fn test_network() {
        let mut builder = crate::graph::GraphBuilder::new();

        let mut one = builder.add_node(box_dyn_call(crate::OneDynCall {}));
        let add = builder.add_node(box_dyn_call(crate::AddDynCall {}));

        one.out_port("value").connect_to(add.in_port("a"));
        one.out_port("value").connect_to(add.in_port("b"));

        // let graph = builder.build();
    }

    fn make_test_graph() -> PODGraph {
        let mut graph = PODGraph::default();

        graph.nodes.push(Node {
            id: uuid::Uuid::new_v4().to_string(),
            kind: "one".into(),
            incoming_connections: vec![],
            data: None,
        });
        graph.nodes.push(Node {
            id: uuid::Uuid::new_v4().to_string(),
            kind: "add".into(),
            incoming_connections: vec![
                Connection {
                    from_id: graph.nodes[0].id.clone(),
                    from_port: "value".to_string(),
                    to_port: "a".to_string(),
                },
                Connection {
                    from_id: graph.nodes[0].id.clone(),
                    from_port: "value".to_string(),
                    to_port: "b".to_string(),
                },
            ],
            data: None,
        });
        graph
    }

    #[test]
    fn test_experimental_pod() {
        let graph = make_test_graph();

        let sorted = pod_to_sorted(&graph).unwrap();
        assert_eq!(sorted.sort, vec![0, 1]);
    }

    struct TestFactory;
    impl NodeFactory for TestFactory {
        fn create(&self, node: &Node) -> anyhow::Result<Box<dyn DynCall>> {
            match node.kind.as_str() {
                "one" => Ok(box_dyn_call(crate::OneDynCall {})),
                "add" => Ok(box_dyn_call(crate::AddDynCall {})),
                _ => anyhow::bail!("Unknown node kind"),
            }
        }
    }

    #[test]
    fn test_builder() {
        let mut builder = GraphBuilder::new();
        let one = builder.add_node("one");
        let mut add = builder.add_node("add");
        one.out_port("value").connect_to(&add.in_port("a"));
        one.out_port("value").connect_to(&add.in_port("b"));

        let graph = builder.build();
        let sorted = pod_to_sorted(&graph).unwrap();
        assert_eq!(sorted.sort, vec![0, 1]);

        let mut _exec = sorted_to_exec(&sorted, TestFactory{}).unwrap();
        //  let count = exec.run().unwrap();
        //  assert_eq!(count, 2);
    }
}
