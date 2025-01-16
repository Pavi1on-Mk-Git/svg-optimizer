use super::EasyIter;
use crate::node::Node;
use crate::node::Node::RegularNode;
use anyhow::Result;
use itertools::Itertools;
use xml::attribute::OwnedAttribute;

fn remove_attr_whitespace_from_node(node: Node) -> Node {
    match node {
        RegularNode {
            node_type,
            attributes,
            children,
        } => RegularNode {
            node_type,
            attributes: attributes.map(|OwnedAttribute { name, value }| OwnedAttribute {
                name,
                value: value.split_whitespace().join(" "),
            }),
            children: children.map(remove_attr_whitespace_from_node),
        },
        other => other,
    }
}

pub fn remove_attr_whitespace(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.map(remove_attr_whitespace_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_attr_whitespace,
        remove_attr_whitespace,
        "<svg xmlns=\"http://www.w3.org/2000/svg\">
        <path d=\"M150        5 L75 \n200    L225\t 200 Z      \"/>
        </svg>",
        r#"<svg xmlns="http://www.w3.org/2000/svg">
        <path d="M150 5 L75 200 L225 200 Z"/>
        </svg>
        "#
    );
}
