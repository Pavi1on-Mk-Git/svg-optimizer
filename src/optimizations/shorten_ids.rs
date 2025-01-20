use super::common::constants::*;
use super::common::id::find_ids_for_subtree;
use super::common::iter::EasyIter;
use crate::node::{ChildlessNodeType, Node, RegularNodeType};
use anyhow::Result;
use itertools::Itertools;
use std::collections::BTreeMap;
use xml::attribute::OwnedAttribute;

struct IdGenerator {
    base_characters: Vec<char>,
    generated_ids: usize,
}

impl IdGenerator {
    fn new() -> Self {
        Self {
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

fn shorten_id_in_attribute(
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
                if attribute.value.contains(&format!("url(#{id})")) {
                    attribute.value = attribute
                        .value
                        .replace(&format!("url(#{id})"), &format!("url(#{new_id})"));
                }
            }
        }
    }
    attribute
}

fn shorten_id_in_text(mut text: String, id_map: &BTreeMap<String, String>) -> String {
    for (old_id, new_id) in id_map {
        text = text.replace(&format!("#{old_id}"), &format!("#{new_id}"));
    }
    text
}

fn shorten_id_in_css(style_child: Node, id_map: &BTreeMap<String, String>) -> Node {
    match style_child {
        Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(text, is_cdata),
        } => Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(shorten_id_in_text(text, id_map), is_cdata),
        },
        other => other,
    }
}

fn shorten_ids_for_node(node: Node, id_map: &BTreeMap<String, String>) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => {
            let shorten_func = if let RegularNodeType::Style = node_type {
                shorten_id_in_css
            } else {
                shorten_ids_for_node
            };

            Node::RegularNode {
                node_type,
                namespace,
                attributes: attributes
                    .map_to_vec(|attribute| shorten_id_in_attribute(attribute, id_map)),
                children: children.map_to_vec(|child| shorten_func(child, id_map)),
            }
        }
        other => other,
    }
}

fn is_hex_color_prefix(id: &str) -> bool {
    id.chars()
        .all(|char| "abcdefABCDEF0123456789".chars().contains(&char))
        && id.len() <= 6
}

fn make_id_map(nodes: &Vec<Node>) -> BTreeMap<String, String> {
    let ids = find_ids_for_subtree(nodes).filter_to_vec(|id| !is_hex_color_prefix(id));

    BTreeMap::from_iter(
        ids.clone()
            .into_iter()
            .zip(IdGenerator::new().filter(|id| !ids.contains(id))),
    )
}

pub fn shorten_ids(nodes: Vec<Node>) -> Result<Vec<Node>> {
    let id_map = make_id_map(&nodes);
    Ok(nodes.map_to_vec(|node| shorten_ids_for_node(node, &id_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    #[test]
    fn test_id_generation() {
        let chars_size = 40;
        let mut gen = IdGenerator::new();

        assert_eq!(gen.nth(5), Some("l".into()));
        assert_eq!(gen.nth(chars_size - 1), Some("lg".into()));
        assert_eq!(gen.nth(chars_size.pow(2)), Some("mgg".into()));
        assert_eq!(
            gen.nth(chars_size.pow(3) - chars_size - 1),
            Some("mZZ".into())
        );
    }

    test_optimize!(
        test_shorten_ids_with_new_id_existing,
        shorten_ids,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100" height="100">
            <rect id="nestedRect" x="10" y="10" width="100" height="100"/>
        </rect>
        <rect id="mediumRect" x="10" y="10" width="100" height="100"/>
        <rect id="largeRect" x="10" y="10" width="100" height="100"/>
        <rect id="g" x="10" y="10" width="100" height="100"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="h" x="10" y="10" width="100" height="100">
            <rect id="i" x="10" y="10" width="100" height="100"/>
        </rect>
        <rect id="j" x="10" y="10" width="100" height="100"/>
        <rect id="k" x="10" y="10" width="100" height="100"/>
        <rect id="l" x="10" y="10" width="100" height="100"/>
        </svg>
        "#
    );

    test_optimize!(
        test_shorten_id_references,
        shorten_ids,
        r##"
        <svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
        <style>
            #smallRect {
                stroke: #000066;
                fill: #00cc00;
            }
            #unused {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#smallRect" x="10" fill="blue" />
        <use href="#unused" x="10" fill="blue" />
        <rect id="smallRect" x="10" y="10" width="100" height="100" />
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            #g {
                stroke: #000066;
                fill: #00cc00;
            }
            #unused {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#g" x="10" fill="blue"/>
        <use href="#unused" x="10" fill="blue"/>
        <rect id="g" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );

    test_optimize!(
        test_shorten_id_references_cdata,
        shorten_ids,
        r##"
        <svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
        <style>
            <![CDATA[
            #smallRect {
                stroke: #000066;
                fill: #00cc00;
            }
            ]]>
        </style>

        <use href="#smallRect" x="10" fill="blue" />
        <rect id="smallRect" x="10" y="10" width="100" height="100" />
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            <![CDATA[
            #g {
                stroke: #000066;
                fill: #00cc00;
            }
            ]]>
        </style>

        <use href="#g" x="10" fill="blue"/>
        <rect id="g" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );

    test_optimize!(
        test_shorten_id_same_as_hex_color_not_shortened,
        shorten_ids,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            <![CDATA[
            #aacc00 {
                stroke: #000066;
                fill: #aacc00;
            }
            ]]>
        </style>

        <use href="#aacc00" x="10" fill="blue"/>
        <rect id="aacc00" x="10" y="10" width="100" height="100"/>
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            <![CDATA[
            #aacc00 {
                stroke: #000066;
                fill: #aacc00;
            }
            ]]>
        </style>

        <use href="#aacc00" x="10" fill="blue"/>
        <rect id="aacc00" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );

    test_optimize!(
        test_shorten_id_used_in_attribute_via_url,
        shorten_ids,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <defs>
            <linearGradient id="grad">
                <stop stop-color="#9a9582" offset="0"/>
                <stop stop-color="#adaa9f" offset="1"/>
            </linearGradient>
        </defs>

        <rect fill="url(#grad)" x="10" y="10" width="100" height="100"/>
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <defs>
            <linearGradient id="g">
                <stop stop-color="#9a9582" offset="0"/>
                <stop stop-color="#adaa9f" offset="1"/>
            </linearGradient>
        </defs>

        <rect fill="url(#g)" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );
}
