use super::common::iter::EasyIter;
use crate::node::{ChildlessNodeType, Node, RegularNodeType};

fn remove_whitespace_outside_tags_from_node(node: Node) -> Option<Node> {
    match node {
        text @ Node::RegularNode {
            node_type: RegularNodeType::Text,
            ..
        } => Some(text),
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: remove_whitespace_outside_tags(children),
        }),
        Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(text, is_cdata),
        } => (!text.trim().is_empty()).then_some(Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(text, is_cdata),
        }),

        other => Some(other),
    }
}

pub(crate) fn remove_whitespace_outside_tags(nodes: Vec<Node>) -> Vec<Node> {
    nodes.filter_map_to_vec(remove_whitespace_outside_tags_from_node)
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
        test_remove_whitespace_outside_tags_cdata,
        remove_whitespace_outside_tags,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <![CDATA[  ]]>
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
