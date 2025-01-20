use super::common::iter::EasyIter;
use crate::node::{ChildlessNodeType, Node};

fn remove_doctype_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: remove_doctype(children),
        }),
        Node::ChildlessNode {
            node_type: ChildlessNodeType::ProcessingInstruction(_, _),
        } => None,
        childless_node => Some(childless_node),
    }
}

pub fn remove_doctype(nodes: Vec<Node>) -> Vec<Node> {
    nodes.filter_map_to_vec(remove_doctype_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_doctype,
        remove_doctype,
        r#"
        <!-- comment --><!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
        <svg xmlns="http://www.w3.org/2000/svg"><!-- comment --></svg>
        <!-- comment -->
        "#,
        r#"
        <!-- comment -->
        <svg xmlns="http://www.w3.org/2000/svg"><!-- comment --></svg>
        <!-- comment -->
        "#
    );
}
