use super::find_ids_for_subtree;
use super::EasyIter;
use crate::node::{ChildlessNodeType, Node, RegularNodeType};
use anyhow::Result;
use std::collections::BTreeMap;
use xml::attribute::OwnedAttribute;

struct IdGenerator {
    base_characters: Vec<char>,
    generated_ids: usize,
}

impl IdGenerator {
    fn new() -> Self {
        Self {
            base_characters: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
                .chars()
                .collect(),
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

        Some(char_ids.map(|id| self.base_characters[id]))
    }
}

fn shorten_id_in_attribute(
    mut attribute: OwnedAttribute,
    id_map: &BTreeMap<String, String>,
) -> OwnedAttribute {
    if attribute.name.local_name == "id" {
        attribute.value = id_map[&attribute.value].clone();
    }

    if attribute.name.local_name == "href" {
        let (first, rest) = attribute.value.split_at(1);
        if first == "#" {
            if let Some(new_id) = id_map.get(rest) {
                attribute.value = format!("#{}", new_id);
            }
        }
    }

    attribute
}

fn shorten_id_in_css(style_child: Node, id_map: &BTreeMap<String, String>) -> Node {
    if let Node::ChildlessNode {
        node_type: ChildlessNodeType::Text(mut text),
    } = style_child
    {
        for (old_id, new_id) in id_map {
            text = text.replace(&format!("#{}", old_id), &format!("#{}", new_id));
        }
        Node::ChildlessNode {
            node_type: ChildlessNodeType::Text(text),
        }
    } else {
        style_child
    }
}

fn shorten_ids_for_node(node: Node, id_map: &BTreeMap<String, String>) -> Node {
    match node {
        Node::RegularNode {
            node_type,
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
                attributes: attributes.map(|attribute| shorten_id_in_attribute(attribute, id_map)),
                children: children.map(|child| shorten_func(child, id_map)),
            }
        }
        other => other,
    }
}

fn make_id_map(nodes: &Vec<Node>) -> BTreeMap<String, String> {
    let ids = find_ids_for_subtree(nodes);
    BTreeMap::from_iter(ids.into_iter().zip(IdGenerator::new()))
}

pub fn shorten_ids(nodes: Vec<Node>) -> Result<Vec<Node>> {
    let id_map = make_id_map(&nodes);
    Ok(nodes.map(|node| shorten_ids_for_node(node, &id_map)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    #[test]
    fn test_id_generation() {
        let chars_size = 62;
        let mut gen = IdGenerator::new();

        assert_eq!(gen.nth(5), Some("f".into()));
        assert_eq!(gen.nth(chars_size - 1), Some("fa".into()));
        assert_eq!(gen.nth(chars_size.pow(2)), Some("gaa".into()));
        assert_eq!(
            gen.nth(chars_size.pow(3) - chars_size - 1),
            Some("g99".into())
        );
    }

    test_optimize!(
        test_shorten_ids,
        shorten_ids,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100" height="100">
            <rect id="nestedRect" x="10" y="10" width="100" height="100"/>
        </rect>
        <rect id="mediumRect" x="10" y="10" width="100" height="100"/>
        <rect id="largeRect" x="10" y="10" width="100" height="100"/>
        <rect id="hugeRect" x="10" y="10" width="100" height="100"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="a" x="10" y="10" width="100" height="100">
            <rect id="b" x="10" y="10" width="100" height="100"/>
        </rect>
        <rect id="c" x="10" y="10" width="100" height="100"/>
        <rect id="d" x="10" y="10" width="100" height="100"/>
        <rect id="e" x="10" y="10" width="100" height="100"/>
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
        <svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
        <style>
            #a {
                stroke: #000066;
                fill: #00cc00;
            }
            #unused {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#a" x="10" fill="blue"/>
        <use href="#unused" x="10" fill="blue"/>
        <rect id="a" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );
}
