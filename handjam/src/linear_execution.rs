use crate::descriptive_ive::{SortedGraph, Node};

trait StoreAllocation {
    fn kind(&self) -> &str;
}
trait ExecutionNode {
    type IndexIterator: Iterator<Item = usize>;
    fn kind(&self) -> &str;
    fn input_indices(&self) -> Self::IndexIterator;
    fn output_indices(&self) -> Self::IndexIterator;
}

// struct LinearExecutionDesc;
// impl LinearExecutionDesc {
//     fn store_allocations(&self) -> impl Iterator<Item = dyn StoreAllocation> {
//     }
//     fn execution_nodes(&self) -> impl Iterator<Item = dyn ExecutionNode> {
//     }
// }

pub struct NodeDesc<'a> {
    inputs: &'a [&'a str],
    outputs: &'a [&'a str],
}

pub trait NodeDatabase {
    fn lookup<'a>(&self, kind: &str) -> anyhow::Result<NodeDesc<'a>>;
}

struct FakeDB;
impl NodeDatabase for FakeDB {
    fn lookup<'a>(&self, kind: &str) -> anyhow::Result<NodeDesc<'a>> {
        let ret = match kind {
            "one" => NodeDesc {
                inputs: &[],
                outputs: &["value"],
            },
            "add" => NodeDesc {
                inputs: &["a", "b"],
                outputs: &["value"],
            },
            _ => return Err(anyhow::anyhow!("Unknown kind: {}", kind)),
        };
        Ok(ret)
    }
}

pub struct LinearExecutionDesc<'a> {
    pub description: Vec<NodeDesc<'a>>,
    pub input_mapping: Vec<Vec<usize>>,
}

impl<'a> LinearExecutionDesc<'a> {
    pub fn new(sorted_graph: &'a SortedGraph, db: impl NodeDatabase + 'a) -> anyhow::Result<LinearExecutionDesc<'a>> {
        let sorted_nodes = sorted_graph.iter().collect::<Vec<_>>();

        let lookup = |n : &Node| {
            let res = db.lookup(&n.kind);
            res
        };

        // Lookup each sorted node into a description from the database.
        // This can fail and will be bailed early if an entity isn't in the database.
        let descs = sorted_nodes
            .iter()
            .map(|n| lookup(n))
            .collect::<anyhow::Result<Vec<NodeDesc>>>()?;

        // Zip these two guys up, and map inputs to outputs.
        let mut node_input_mapping = vec![];
        for (node, desc) in sorted_nodes.iter().zip(descs.iter()) {
            let connections = &node.incoming_connections;
            let mut this_node_mapping = vec![];
            for input in desc.inputs.iter() {
                // find the input name in the connection list
                let thisconnection = connections.iter().find(|c| c.to_port == *input).ok_or_else(|| anyhow::anyhow!("Missing connection for input {}", input))?;
                // find the output node
                let output_node_index = sorted_nodes.iter().position(|n| n.id == thisconnection.from_id).ok_or_else(|| anyhow::anyhow!("Could not find source node for connection {}", input))?;
                let output_desc = &descs[output_node_index];
                // find the output port index
                let output_port_index = output_desc.outputs.iter().position(|p| p == &thisconnection.from_port).ok_or_else(|| anyhow::anyhow!("Could not find source port for connection {}", input))?;
                this_node_mapping.push((output_node_index, output_port_index));
            }
            node_input_mapping.push(this_node_mapping);
        }

        // Generate an accumulated list of allocations, adding up the length of each allocation.
        // This is awesome blackmagic iterator stuff.
        let allocation_offsets = descs.iter().fold(vec![], |mut acc, desc| {
            let last = acc.last().unwrap_or(&0);
            acc.push(last + desc.outputs.len());
            acc
        });

        // redo the node_input_mappings to go from (output_node_index,output_port_index) to absolute allocation offset
        let input_mapping = node_input_mapping.iter().map(|node_mapping| {
            node_mapping.iter().map(|(output_node_index, output_port_index)| {
                allocation_offsets[*output_node_index] + output_port_index
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();

        Ok(LinearExecutionDesc {
            description: descs,
            input_mapping
        })
    }
}
