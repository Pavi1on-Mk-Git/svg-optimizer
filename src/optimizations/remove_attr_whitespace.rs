use itertools::Itertools;
use xml::attribute::OwnedAttribute;

use crate::node::Node;
use crate::node::Node::RegularNode;

use super::apply_to_nodes;

fn remove_attr_whitespace_from_node(node: Node) -> Option<Node> {
    Some(match node {
        RegularNode {
            node_type,
            attributes,
            children,
        } => RegularNode {
            node_type,
            attributes: attributes
                .into_iter()
                .map(|OwnedAttribute { name, value }| OwnedAttribute {
                    name,
                    value: value.split_whitespace().join(" "),
                })
                .collect(),
            children: remove_attr_whitespace(children),
        },
        other => other,
    })
}

pub fn remove_attr_whitespace(nodes: Vec<Node>) -> Vec<Node> {
    apply_to_nodes(nodes, remove_attr_whitespace_from_node)
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
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <path d=\"M150        5 L75 \n200    L225\t 200 Z      \"/>\
        </svg>\
        ",
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <path d=\"M150 5 L75 200 L225 200 Z\"/>\
        </svg>\
        "
    );
}
