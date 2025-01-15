use super::apply_option;
use crate::node::ChildlessNodeType;
use crate::node::Node;
use anyhow::Result;

fn remove_doctype_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            attributes,
            children: remove_doctype(children).unwrap(),
        }),
        Node::ChildlessNode {
            node_type: ChildlessNodeType::ProcessingInstruction(_, _),
        } => None,
        childless_node => Some(childless_node),
    }
}

pub fn remove_doctype(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(apply_option(nodes, remove_doctype_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::test::test_optimize;
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
