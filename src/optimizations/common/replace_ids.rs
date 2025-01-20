use super::{constants::*, iter::EasyIter};
use crate::node::{ChildlessNodeType, Node, RegularNodeType};
use std::collections::BTreeMap;
use xml::attribute::OwnedAttribute;

pub struct IdGenerator {
    base_characters: Vec<char>,
    generated_ids: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            // not using abcdef to avoid conflicts with hex colors in CSS
            base_characters: "ghijklmnopqrstuvwxyzGHIJKLMNOPQRSTUVWXYZ".chars().collect(),
            generated_ids: 0,
        }
    }
}

impl Iterator for IdGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut char_ids = vec![];
        let mut remaining_count = self.generated_ids;

        loop {
            char_ids.push(remaining_count % self.base_characters.len());
            remaining_count /= self.base_characters.len();

            if remaining_count == 0 {
                break;
            }
            remaining_count -= 1;
        }

        self.generated_ids += 1;

        Some(char_ids.map_to_vec(|id| self.base_characters[id]))
    }
}

fn replace_ids_in_attribute(
    mut attribute: OwnedAttribute,
    id_map: &BTreeMap<String, String>,
) -> OwnedAttribute {
    if attribute.name.local_name == ID_NAME {
        if let Some(new_id) = id_map.get(&attribute.value) {
            attribute.value = new_id.clone();
        }
    }

    match attribute.name.local_name.as_str() {
        HREF_NAME => {
            let (first, rest) = attribute.value.split_at(1);
            if first == "#" {
                if let Some(new_id) = id_map.get(rest) {
                    attribute.value = format!("#{new_id}");
                }
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
        other => other,
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
        other => other,
    }
}

pub fn replace_ids(nodes: Vec<Node>, id_map: &BTreeMap<String, String>) -> Vec<Node> {
    nodes.map_to_vec(|node| replace_ids_for_node(node, id_map))
}
