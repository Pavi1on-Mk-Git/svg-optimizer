use super::common::constants::*;
use super::common::id::make_id_usage_map;
use super::common::iter::EasyIter;
use crate::node::Node;
use anyhow::Result;
use std::collections::BTreeMap;
use xml::attribute::OwnedAttribute;

fn is_attribute_useless_id(
    attribute: &OwnedAttribute,
    id_usage_map: &BTreeMap<String, bool>,
) -> bool {
    attribute.name.local_name == ID_NAME && !id_usage_map[&attribute.value]
}

fn remove_useless_ids_for_node(node: Node, id_usage_map: &BTreeMap<String, bool>) -> Node {
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
                .filter(|attribute| !is_attribute_useless_id(attribute, id_usage_map)),
            children: children.map(|child| remove_useless_ids_for_node(child, id_usage_map)),
        },
        other => other,
    }
}

pub fn remove_useless_ids(nodes: Vec<Node>) -> Result<Vec<Node>> {
    let id_usage_map = make_id_usage_map(&nodes);
    Ok(nodes.map(|node| remove_useless_ids_for_node(node, &id_usage_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_useless_ids,
        remove_useless_ids,
        r##"
        <svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
        <style>
            #smallRect {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#bigRect" x="10" fill="blue"/>
        <rect id="smallRect" x="10" y="10" width="100" height="100"/>
        <rect id="bigRect" x="10" y="10" width="100" height="100"/>
        <rect id="thirdRect" x="10" y="10" width="100" height="100"/>
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            #smallRect {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#bigRect" x="10" fill="blue"/>
        <rect id="smallRect" x="10" y="10" width="100" height="100"/>
        <rect id="bigRect" x="10" y="10" width="100" height="100"/>
        <rect x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );
}
