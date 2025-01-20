use super::common::iter::EasyIter;
use crate::node::Node;
use anyhow::Result;
use lazy_regex::regex_replace_all;
use xml::attribute::OwnedAttribute;

fn round_floating_point_numbers_in_attribute(
    mut attr: OwnedAttribute,
    precision: u32,
) -> OwnedAttribute {
    attr.value = regex_replace_all!(
        r"[+-]?\d*\.\d+([Ee]\d+)?",
        attr.value.as_str(),
        |float: &str, _| {
            let mult = 10u32.pow(precision) as f64;
            let rounded = (float.parse::<f64>().unwrap() * mult).round() / mult;
            format!("{}", rounded)
        }
    )
    .into_owned();
    attr
}

fn round_floating_point_numbers_in_node(node: Node, precision: u32) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes: attributes
                .map_to_vec(|attr| round_floating_point_numbers_in_attribute(attr, precision)),
            children: children
                .map_to_vec(|child| round_floating_point_numbers_in_node(child, precision)),
        },
        other => other,
    }
}

pub fn round_floating_point_numbers(nodes: Vec<Node>, precision: u32) -> Result<Vec<Node>> {
    Ok(nodes.map_to_vec(|node| round_floating_point_numbers_in_node(node, precision)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    fn test_round(nodes: Vec<Node>) -> Result<Vec<Node>> {
        round_floating_point_numbers(nodes, 2)
    }

    test_optimize!(
        test_round_floating_point_numbers,
        test_round,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
        <path d="M 10,30.157 A 20,20.123 0,0,1 50,3.1e1 A 20.301,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 1.9E1,30 z"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
        <path d="M 10,30.16 A 20,20.12 0,0,1 50,31 A 20.3,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 19,30 z"/>
        </svg>
        "#
    );
}
