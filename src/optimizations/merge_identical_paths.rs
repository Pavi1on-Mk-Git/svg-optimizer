// use super::common::{
//     id_usage::find_ids_for_subtree,
//     iter::EasyIter,
//     replace_ids::{replace_ids, IdGenerator},
// };
use crate::node::Node;
// use itertools::Itertools;
// use xml::attribute::OwnedAttribute;

// fn is_hex_color_prefix(id: &str) -> bool {
//     id.chars()
//         .all(|char| "abcdefABCDEF0123456789".chars().contains(&char))
//         && id.len() <= 6
// }

// fn make_shorten_ids_map(nodes: &Vec<Node>) -> BTreeMap<String, String> {
//     let ids = find_ids_for_subtree(nodes).filter_to_vec(|id| !is_hex_color_prefix(id));

//     BTreeMap::from_iter(
//         ids.clone()
//             .into_iter()
//             .zip(IdGenerator::new().filter(|id| !ids.contains(id))),
//     )
// }

// fn remove_attribute_whitespace_from_node(node: Node) -> Node {
//     match node {
//         Node::RegularNode {
//             node_type,
//             namespace,
//             attributes,
//             children,
//         } => Node::RegularNode {
//             node_type,
//             namespace,
//             attributes: attributes.map_to_vec(|OwnedAttribute { name, value }| OwnedAttribute {
//                 name,
//                 value: value.split_whitespace().join(" "),
//             }),
//             children: merge_identical_paths(children),
//         },
//         other => other,
//     }
// }

pub fn merge_identical_paths(nodes: Vec<Node>) -> Vec<Node> {
    nodes
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::optimizations::common::test::test_optimize;
//     use crate::parser::Parser;
//     use crate::writer::SVGWriter;

//     test_optimize!(
//         test_remove_attr_whitespace,
//         merge_identical_paths,
//         "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"  0  0\n   100    100\">
//         <path d=\"M150        5 L75 \n200    L225\t 200 Z      \"/>
//         </svg>",
//         r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
//         <path d="M150 5 L75 200 L225 200 Z"/>
//         </svg>
//         "#
//     );
// }
