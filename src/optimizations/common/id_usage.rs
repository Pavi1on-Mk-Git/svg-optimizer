use super::constants::{HREF_NAME, ID_NAME};
use crate::node::{ChildlessNodeType, Node, RegularNodeType};
use std::collections::BTreeMap;
use std::iter::repeat;
use xml::attribute::OwnedAttribute;

pub(crate) fn find_attribute<'a>(
    attributes: &'a [OwnedAttribute],
    name: &str,
) -> Option<&'a String> {
    attributes
        .iter()
        .find(|attr| attr.name.local_name == name)
        .map(|id| &id.value)
}

pub(crate) fn find_attribute_mut<'a>(
    attributes: &'a mut [OwnedAttribute],
    name: &str,
) -> Option<&'a mut String> {
    attributes
        .iter_mut()
        .find(|attr| attr.name.local_name == name)
        .map(|id| &mut id.value)
}

pub(crate) fn find_ids_for_subtree(nodes: &Vec<Node>) -> Vec<String> {
    let mut ids = vec![];

    for node in nodes {
        if let Node::RegularNode {
            attributes,
            children,
            ..
        } = node
        {
            if let Some(id) = find_attribute(attributes, ID_NAME) {
                ids.push(id.clone());
            }

            ids.extend(find_ids_for_subtree(children));
        }
    }

    ids
}

fn find_id_usage_in_attribute(attribute: &OwnedAttribute, id_map: &mut BTreeMap<String, bool>) {
    match attribute.name.local_name.as_str() {
        HREF_NAME => {
            let (first, rest) = attribute.value.split_at(1);
            if first == "#" {
                if let Some(value_in_map) = id_map.get_mut(rest) {
                    *value_in_map = true;
                }
            }
        }
        _ => {
            for (id, value_in_map) in id_map.iter_mut() {
                if attribute.value.contains(&format!("url(#{id})")) {
                    *value_in_map = true;
                }
            }
        }
    }
}

fn find_id_usages_in_style_node(style_child: &Node, id_map: &mut BTreeMap<String, bool>) {
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
        ..
    } = node
    {
        attributes
            .iter()
            .for_each(|attribute| find_id_usage_in_attribute(attribute, id_map));

        let find_func = if let RegularNodeType::Style = node_type {
            find_id_usages_in_style_node
        } else {
            find_id_usages_for_node
        };

        children.iter().for_each(|child| find_func(child, id_map));
    }
}

pub(crate) fn make_id_usage_map(nodes: &Vec<Node>) -> BTreeMap<String, bool> {
    let ids = find_ids_for_subtree(nodes);
    let mut id_usage_map = ids.into_iter().zip(repeat(false)).collect();

    nodes
        .iter()
        .for_each(|node| find_id_usages_for_node(node, &mut id_usage_map));

    id_usage_map
}
