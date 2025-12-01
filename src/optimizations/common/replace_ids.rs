use super::{
    constants::{HREF_NAME, ID_NAME},
    iter::EasyIter,
};
use crate::node::{ChildlessNodeType, Node, RegularNodeType};
use std::collections::BTreeMap;
use xml::attribute::OwnedAttribute;

fn replace_ids_in_attribute(
    mut attribute: OwnedAttribute,
    id_map: &BTreeMap<String, String>,
) -> OwnedAttribute {
    if attribute.name.local_name == ID_NAME
        && let Some(new_id) = id_map.get(&attribute.value)
    {
        attribute.value = new_id.clone();
    }

    match attribute.name.local_name.as_str() {
        HREF_NAME => {
            if let Some((first, rest)) = attribute.value.split_once('#')
                && first == ""
                && let Some(new_id) = id_map.get(rest)
            {
                attribute.value = format!("#{new_id}");
            }
        }
        _ => {
            for (id, new_id) in id_map {
                attribute.value = attribute
                    .value
                    .replace(&format!("url(#{id})"), &format!("url(#{new_id})"));
            }
        }
    }
    attribute
}

fn replace_ids_in_text(mut text: String, id_map: &BTreeMap<String, String>) -> String {
    for (old_id, new_id) in id_map {
        text = text.replace(&format!("#{old_id}"), &format!("#{new_id}"));
    }
    text
}

fn replace_ids_in_css(style_child: Node, id_map: &BTreeMap<String, String>) -> Node {
    match style_child {
        Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(text, is_cdata),
        } => Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(replace_ids_in_text(text, id_map), is_cdata),
        },
        other @ (Node::RegularNode { .. } | Node::ChildlessNode { .. }) => other,
    }
}

fn replace_ids_for_node(node: Node, id_map: &BTreeMap<String, String>) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => {
            let shorten_func = if let RegularNodeType::Style = node_type {
                replace_ids_in_css
            } else {
                replace_ids_for_node
            };

            Node::RegularNode {
                node_type,
                namespace,
                attributes: attributes
                    .map_to_vec(|attribute| replace_ids_in_attribute(attribute, id_map)),
                children: children.map_to_vec(|child| shorten_func(child, id_map)),
            }
        }
        other @ Node::ChildlessNode { .. } => other,
    }
}

pub(crate) fn replace_ids(nodes: Vec<Node>, id_map: &BTreeMap<String, String>) -> Vec<Node> {
    nodes.map_to_vec(|node| replace_ids_for_node(node, id_map))
}
