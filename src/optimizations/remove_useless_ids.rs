use super::common::constants::*;
use super::common::helpers::find_ids_for_subtree;
use super::common::iter::EasyIter;
use crate::node::{ChildlessNodeType, Node, RegularNodeType};
use anyhow::Result;
use std::collections::BTreeMap;
use std::iter::repeat;
use xml::attribute::OwnedAttribute;

fn find_id_usage_in_attribute(attribute: &OwnedAttribute, id_map: &mut BTreeMap<String, bool>) {
    if attribute.name.local_name == HREF_NAME {
        let (first, rest) = attribute.value.split_at(1);
        if first == "#" {
            if let Some(value_in_map) = id_map.get_mut(rest) {
                *value_in_map = true;
            }
        }
    }
}

fn find_id_usages_in_css(style_child: &Node, id_map: &mut BTreeMap<String, bool>) {
    if let Node::ChildlessNode {
        node_type: ChildlessNodeType::Text(text, ..),
    } = style_child
    {
        for (id, value_in_map) in id_map.iter_mut() {
            if text.contains(&format!("#{id}")) {
                *value_in_map = true;
            }
        }
    }
}

fn find_id_usages_for_node(node: &Node, id_map: &mut BTreeMap<String, bool>) {
    if let Node::RegularNode {
        node_type,
        attributes,
        children,
    } = node
    {
        attributes
            .iter()
            .for_each(|attribute| find_id_usage_in_attribute(attribute, id_map));

        let find_func = if let RegularNodeType::Style = node_type {
            find_id_usages_in_css
        } else {
            find_id_usages_for_node
        };

        children.iter().for_each(|child| find_func(child, id_map));
    }
}

fn make_id_usage_map(nodes: &Vec<Node>) -> BTreeMap<String, bool> {
    let ids = find_ids_for_subtree(nodes);
    let mut id_usage_map = BTreeMap::from_iter(ids.into_iter().zip(repeat(false)));

    nodes
        .iter()
        .for_each(|node| find_id_usages_for_node(node, &mut id_usage_map));

    id_usage_map
}

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
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
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
