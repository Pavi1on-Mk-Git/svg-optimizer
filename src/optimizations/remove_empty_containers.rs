use super::common::iter::EasyIter;
use crate::node::{Node, RegularNodeType};

fn remove_empty_containers_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type:
                node_type @ (RegularNodeType::Anchor
                | RegularNodeType::Defs
                | RegularNodeType::Group
                | RegularNodeType::Marker
                | RegularNodeType::Mask
                | RegularNodeType::Pattern
                | RegularNodeType::Svg
                | RegularNodeType::Switch
                | RegularNodeType::Symbol),
            namespace,
            attributes,
            children,
        } => {
            let new_children: Vec<Node> = remove_empty_containers(children);

            match new_children.len() {
                0 => None,
                _ => Some(Node::RegularNode {
                    node_type,
                    namespace,
                    attributes,
                    children: new_children,
                }),
            }
        }
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: remove_empty_containers(children),
        }),
        other => Some(other),
    }
}

pub(crate) fn remove_empty_containers(nodes: Vec<Node>) -> Vec<Node> {
    nodes.filter_map_to_vec(remove_empty_containers_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_empty_containers,
        remove_empty_containers,
        r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>
        "#,
        r"
        "
    );

    test_optimize!(
        test_remove_empty_containers_multiple,
        remove_empty_containers,
        r#"<svg xmlns="http://www.w3.org/2000/svg"><a><g></g></a><defs></defs></svg>
        "#,
        r"
        "
    );
}
