use super::common::iter::EasyIter;
use crate::node::Node;
use itertools::Itertools;
use xml::attribute::OwnedAttribute;

fn remove_attribute_whitespace_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes: attributes.map_to_vec(|OwnedAttribute { name, value }| OwnedAttribute {
                name,
                value: value.split_whitespace().join(" "),
            }),
            children: remove_attribute_whitespace(children),
        },
        other @ Node::ChildlessNode { .. } => other,
    }
}

pub(crate) fn remove_attribute_whitespace(nodes: Vec<Node>) -> Vec<Node> {
    nodes.map_to_vec(remove_attribute_whitespace_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_attr_whitespace,
        remove_attribute_whitespace,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"  0  0\n   100    100\">
        <path d=\"M150        5 L75 \n200    L225\t 200 Z      \"/>
        </svg>",
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
        <path d="M150 5 L75 200 L225 200 Z"/>
        </svg>
        "#
    );
}
