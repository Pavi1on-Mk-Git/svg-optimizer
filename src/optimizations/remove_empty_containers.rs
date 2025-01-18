use super::common::iter::EasyIter;
use crate::node::{Node, RegularNodeType};
use anyhow::Result;

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
            let new_children: Vec<Node> = children.filter_map(remove_empty_containers_from_node);

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
            children: children.filter_map(remove_empty_containers_from_node),
        }),
        other => Some(other),
    }
}

pub fn remove_empty_containers(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.filter_map(remove_empty_containers_from_node))
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
        r#"
        "#
    );

    test_optimize!(
        test_remove_empty_containers_multiple,
        remove_empty_containers,
        r#"<svg xmlns="http://www.w3.org/2000/svg"><a><g></g></a><defs></defs></svg>
        "#,
        r#"
        "#
    );
}
