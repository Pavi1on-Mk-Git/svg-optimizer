use super::common::{iter::EasyIter, unit::round_float};
use crate::node::Node;
use lazy_regex::regex_replace_all;
use xml::attribute::OwnedAttribute;

fn round_floats_in_attribute(mut attr: OwnedAttribute, precision: usize) -> OwnedAttribute {
    attr.value = regex_replace_all!(
        r"[+-]?\d*\.\d+([Ee]\d+)?",
        attr.value.as_str(),
        |float: &str, _| round_float(float.parse::<f64>().unwrap(), precision)
    )
    .into_owned();
    attr
}

fn round_floats_in_node(node: Node, precision: usize) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes: attributes.map_to_vec(|attr| round_floats_in_attribute(attr, precision)),
            children: children.map_to_vec(|child| round_floats_in_node(child, precision)),
        },
        other => other,
    }
}

pub(crate) fn round_floats(nodes: Vec<Node>, precision: usize) -> Vec<Node> {
    nodes.map_to_vec(|node| round_floats_in_node(node, precision))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    fn test_round(nodes: Vec<Node>) -> Vec<Node> {
        round_floats(nodes, 2)
    }

    test_optimize!(
        test_round_floats,
        test_round,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 -00.101 00100.12">
        <path d="M 10,30.157 A 20,20.123 0,0,1 50,3.1e1 A 20.301,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 1.9E1,30 z"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 -.1 100.12">
        <path d="M 10,30.16 A 20,20.12 0,0,1 50,31 A 20.3,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 19,30 z"/>
        </svg>
        "#
    );
}
