use super::apply_to_nodes;
use crate::node::Node;
use crate::node::Node::RegularNode;
use std::collections::BTreeMap;

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

        Some(
            char_ids
                .into_iter()
                .map(|id| self.base_characters[id])
                .collect(),
        )
    }
}

fn make_id_map(nodes: &Vec<Node>) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut ids = vec![];

    for node in nodes {
        if let RegularNode {
            attributes,
            children,
            ..
        } = node
        {
            if let Some(id) = attributes.iter().find(|attr| attr.name.local_name == "id") {
                ids.push(id.value.clone());
            }
            map.extend(make_id_map(children));
        }
    }

    map.extend(ids.into_iter().zip(IdGenerator::new()));
    map
}

fn shorten_ids_for_node(node: Node, id_map: &BTreeMap<String, String>) -> Option<Node> {
    if let RegularNode {
        node_type,
        attributes,
        children,
    } = node
    {
        let attributes = attributes
            .into_iter()
            .map(|mut attr| {
                if attr.name.local_name == "id" {
                    attr.value = id_map[&attr.value].clone();
                }
                attr
            })
            .collect();

        Some(RegularNode {
            node_type,
            attributes,
            children: shorten_ids(children),
        })
    } else {
        Some(node)
    }
}

pub fn shorten_ids(nodes: Vec<Node>) -> Vec<Node> {
    let id_map = make_id_map(&nodes);
    apply_to_nodes(nodes, |node| shorten_ids_for_node(node, &id_map))
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
        <rect id="smallRect1" x="10" y="10" width="100" height="100"/>
        <rect id="mediumRect" x="10" y="10" width="100" height="100"/>
        <rect id="largeRect" x="10" y="10" width="100" height="100"/>
        <rect id="hugeRect" x="10" y="10" width="100" height="100"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="a" x="10" y="10" width="100" height="100"/>
        <rect id="b" x="10" y="10" width="100" height="100"/>
        <rect id="c" x="10" y="10" width="100" height="100"/>
        <rect id="d" x="10" y="10" width="100" height="100"/>
        </svg>
        "#
    );
}
