use graph_builder::prelude::*;

fn build_graph() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_website_example() {
        let graph: DirectedCsrGraph<usize> = GraphBuilder::new()
            .edges(vec![(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)])
            .build();

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 5);

        assert_eq!(graph.out_degree(1), 2);
        assert_eq!(graph.in_degree(1), 1);

        assert_eq!(graph.out_neighbors(1).as_slice(), &[2, 3]);
        assert_eq!(graph.in_neighbors(1).as_slice(), &[0]);
    }

    struct Connection<'a> {
        from: &'a GraphNode<'a>,
        port: usize,
    }

    struct GraphNode<'a> {
        incoming_connections: Vec<Connection<'a>>,
    }
    impl<'a> GraphNode<'a> {
        fn new() -> Self {
            Self {
                incoming_connections: Vec::new(),
            }
        }
    }

    #[test]
    fn test_own_data() {
        let mut nodes = vec![];

        let nodea = GraphNode::new();
        let nodeb = GraphNode::new();
        // nodeb.incoming_connections.push(Connection {
        //     from: &nodea,
        //     port: 0,
        // });
        nodes.push(nodea);
        nodes.push(nodeb);

        nodes[0].incoming_connections.push(Connection {
            from: &nodes[1],
            port: 0,
        });
    }
}
