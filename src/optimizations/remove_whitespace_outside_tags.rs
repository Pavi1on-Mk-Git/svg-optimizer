use super::common::iter::EasyIter;
use crate::node::Node::RegularNode;
use crate::node::{ChildlessNodeType, Node, RegularNodeType};
use anyhow::Result;

fn remove_whitespace_outside_tags_from_node(node: Node) -> Option<Node> {
    match node {
        text @ RegularNode {
            node_type: RegularNodeType::Text,
            ..
        } => Some(text),
        RegularNode {
            node_type,
            attributes,
            children,
        } => Some(RegularNode {
            node_type,
            attributes,
            children: children.filter_map(remove_whitespace_outside_tags_from_node),
        }),
        Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(text),
        } => (!text.trim().is_empty()).then_some(Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(text),
        }),
        other => Some(other),
    }
}

pub fn remove_whitespace_outside_tags(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.filter_map(remove_whitespace_outside_tags_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_whitespace_outside_tags,
        remove_whitespace_outside_tags,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">

        </svg><!-- comment -->
        "#,
        r#"<svg xmlns="http://www.w3.org/2000/svg"/><!-- comment -->"#
    );

    test_optimize!(
        test_no_remove_whitespace_within_text_tag,
        remove_whitespace_outside_tags,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <text>    </text>
        </svg>
        "#,
        r#"<svg xmlns="http://www.w3.org/2000/svg"><text>    </text></svg>"#
    );
}
