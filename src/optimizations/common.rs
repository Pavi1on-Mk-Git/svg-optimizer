macro_rules! define_remove_childless_node {
    ($fn_name:ident, $node_type:ident) => {
        fn remove_from_node(node: Node) -> Option<Node> {
            match node {
                Node::RegularNode {
                    node_type,
                    attributes,
                    children,
                } => Some(Node::RegularNode {
                    node_type,
                    attributes,
                    children: $fn_name(children).unwrap(),
                }),
                Node::ChildlessNode {
                    node_type: ChildlessNodeType::$node_type(_),
                } => None,
                childless_node => Some(childless_node),
            }
        }

        pub fn $fn_name(nodes: Vec<Node>) -> Result<Vec<Node>> {
            Ok(apply_to_nodes(nodes, remove_from_node))
        }
    };
}

pub(crate) use define_remove_childless_node;
