use petgraph::graph::NodeIndex;
use petgraph::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PortBuilder<NODETYPE, PORTTYPE> {
    dag: Rc<RefCell<Graph<NODETYPE, (PORTTYPE, PORTTYPE)>>>,
    index: NodeIndex,
    port: PORTTYPE,
}
impl<NODETYPE, PORTTYPE> PortBuilder<NODETYPE, PORTTYPE> {
    pub fn connect_to(&mut self, other: InPort<PORTTYPE>) -> &mut Self
    where
        PORTTYPE: Clone,
    {
        self.dag
            .borrow_mut()
            .add_edge(self.index, other.index, (self.port.clone(), other.port));
        self
    }
}
pub struct InPort<PORTTYPE> {
    index: NodeIndex,
    port: PORTTYPE,
}

pub struct NodeBuilder<NODETYPE, PORTTYPE> {
    dag: Rc<RefCell<Graph<NODETYPE, (PORTTYPE, PORTTYPE)>>>,
    index: NodeIndex,
}
impl<NODETYPE, PORTTYPE> NodeBuilder<NODETYPE, PORTTYPE> {
    pub fn in_port(&self, port: PORTTYPE) -> InPort<PORTTYPE> {
        InPort {
            index: self.index,
            port,
        }
    }
    pub fn out_port(&mut self, port: PORTTYPE) -> PortBuilder<NODETYPE, PORTTYPE> {
        PortBuilder {
            dag: self.dag.clone(),
            index: self.index,
            port,
        }
    }
}

pub struct GraphBuilder<NODETYPE, PORTTYPE> {
    dag: Rc<RefCell<Graph<NODETYPE, (PORTTYPE, PORTTYPE)>>>,
}
impl<NODETYPE, PORTTYPE> Default for GraphBuilder<NODETYPE, PORTTYPE> {
    fn default() -> Self {
        Self::new()
    }
}
impl<NODETYPE, PORTTYPE> GraphBuilder<NODETYPE, PORTTYPE> {
    pub fn new() -> Self {
        Self {
            dag: Rc::new(RefCell::new(Graph::<NODETYPE, (PORTTYPE, PORTTYPE)>::new())),
        }
    }
    pub fn add_node(&mut self, node: NODETYPE) -> NodeBuilder<NODETYPE, PORTTYPE> {
        let index = self.dag.borrow_mut().add_node(node);
        NodeBuilder {
            dag: self.dag.clone(),
            index,
        }
    }
    pub fn build(self) -> Graph<NODETYPE, (PORTTYPE, PORTTYPE)> {
        self.dag.take()
    }
    pub fn sort(self) -> Result<Vec<NodeIndex>,Box<dyn std::error::Error>> {
        let dag = self.dag.take();

        let sorted = petgraph::algo::toposort(&dag, None).map_err(|_| "Cycle detected")?;
    
        // Reverse the edges, we need to know where the edges go to, not from
        
        Ok(sorted)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use petgraph::algo::toposort;

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_daggy() {
        use daggy::Dag;

        let mut dag = Dag::<&str, usize>::new();
        let a = dag.add_node("A");
        let b = dag.add_node("B");
        let c = dag.add_node("C");
        let d = dag.add_node("D");
        let e = dag.add_node("E");

        _ = dag.add_edge(a, b, 1);
        _ = dag.add_edge(a, c, 1);
        _ = dag.add_edge(b, d, 1);
        _ = dag.add_edge(c, d, 1);
        _ = dag.add_edge(d, e, 1);

        // topological sort dag
        // let sorted = dag
        //     .topological_sort()
        //     .expect("dag is cyclic")
        //     .iter()
        //     .map(|i| dag[*i])
        //     .collect::<Vec<_>>();
    }

    #[test]
    fn test_petgraph() {
        use petgraph::graph::NodeIndex;
        use petgraph::prelude::*;

        type PortPair = (&'static str, &'static str);
        type IncomingConnection = (NodeIndex, PortPair);

        let mut graph = Graph::<&str, PortPair>::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");

        _ = graph.add_edge(a, b, ("One", "Two"));
        _ = graph.add_edge(a, c, ("One", "Two"));
        _ = graph.add_edge(b, d, ("One", "Two"));
        _ = graph.add_edge(c, d, ("One", "Two"));
        _ = graph.add_edge(d, e, ("One", "Two"));

        let topo = toposort(&graph, None).unwrap();
        let indices = topo.iter().map(|i| i.index()).collect::<Vec<_>>();

        let expected = [0, 1, 2, 3, 4];
        let expected2 = [0, 2, 1, 3, 4];
        assert!(indices == expected || indices == expected2);

        // Invert outgoing edges to incoming edges
        let mut incoming_edges: HashMap<String, IncomingConnection> = HashMap::new();
        graph.node_indices().for_each(|node| {
            graph.edges(node).for_each(|edge| {
                let target = graph[edge.target()];
                let source = edge.source();
                let port_pair = edge.weight();
                incoming_edges.insert(target.to_string(), (source, *port_pair));
            });
        });

        assert!(incoming_edges.get("A").is_none());
        assert!(incoming_edges.get("B").is_some());
        assert!(incoming_edges.get("C").is_some());
        assert!(incoming_edges.get("D").is_some());
        assert!(incoming_edges.get("E").is_some());

        assert!(incoming_edges.get("B").unwrap() == &(a, ("One", "Two")));
    }

    #[test]
    fn test_my_builder() {
        let mut builder = GraphBuilder::<&str, &str>::new();

        let b = builder.add_node("B");
        let mut a = builder.add_node("A");

        a.out_port("outport").connect_to(b.in_port("inport"));

        let graph = builder.build();
        let nodes = graph.node_indices().map(|i| graph[i]).collect::<Vec<_>>();

        assert!(nodes == vec!["A", "B"] || nodes == vec!["B", "A"])
    }
    #[test]
    fn test_more_complex_buidler() {
        let mut builder = GraphBuilder::<&str, &str>::new();

        let mut a = builder.add_node("A");
        let mut b = builder.add_node("B");
        let mut c = builder.add_node("C");
        let mut d = builder.add_node("D");
        let e = builder.add_node("E");

        a.out_port("first").connect_to(b.in_port("inport"));
        a.out_port("second").connect_to(c.in_port("inport"));
        b.out_port("outport").connect_to(d.in_port("first"));
        c.out_port("outport").connect_to(d.in_port("second"));
        d.out_port("outport").connect_to(e.in_port("inport"));

        let graph = builder.build();
        let sorted = toposort(&graph, None).unwrap();

        let sorted_names = sorted.iter().map(|i| graph[*i]).collect::<Vec<_>>();
        let expected = vec!["A", "B", "C", "D", "E"];
        let expected2 = vec!["A", "C", "B", "D", "E"];

        assert!(sorted_names == expected || sorted_names == expected2);
    }

    #[test]
    fn test_cycles_fail() {
        let mut builder = GraphBuilder::<&str, &str>::new();

        let mut a = builder.add_node("A");
        let mut b = builder.add_node("B");

        a.out_port("first").connect_to(b.in_port("inport"));
        b.out_port("second").connect_to(a.in_port("inport"));

        let graph = builder.build();
        let sorted = toposort(&graph, None);

        assert!(sorted.is_err());
    }
}
