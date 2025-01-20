use super::common::{
    constants::{HREF_NAME, ID_NAME},
    id_generator::IdGenerator,
    id_usage::{find_attribute_mut, find_ids_for_subtree},
    iter::EasyIter,
    replace_ids::replace_ids,
};
use crate::node::{Node, NodeNamespace, RegularNodeType};
use std::collections::BTreeMap;
use xml::{attribute::OwnedAttribute, name::OwnedName};

fn are_equal_paths(node1: &Node, node2: &Node) -> bool {
    if let (
        Node::RegularNode {
            node_type: RegularNodeType::Path,
            namespace: namespace1,
            attributes: attributes1,
            children: children1,
        },
        Node::RegularNode {
            node_type: RegularNodeType::Path,
            namespace: namespace2,
            attributes: attributes2,
            children: children2,
        },
    ) = (node1, node2)
    {
        let attributes1 = attributes1.filter_to_vec(|attr| attr.name.local_name != ID_NAME);
        let attributes2 = attributes2.filter_to_vec(|attr| attr.name.local_name != ID_NAME);
        namespace1 == namespace2 && attributes1 == attributes2 && children1 == children2
    } else {
        false
    }
}

fn add_path_usages_in_node(node: &Node, path_usages: &mut Vec<(Node, u32)>) {
    match node {
        node @ Node::RegularNode {
            node_type: RegularNodeType::Path,
            ..
        } => {
            if let Some((_, count)) = path_usages
                .iter_mut()
                .find(|(path, _)| are_equal_paths(path, node))
            {
                *count += 1;
            } else {
                path_usages.push((node.clone(), 1));
            }
        }
        Node::RegularNode { children, .. } => {
            for child in children {
                add_path_usages_in_node(child, path_usages);
            }
        }
        _ => {}
    }
}

fn find_path_usages(nodes: &[Node]) -> Vec<(Node, u32)> {
    let mut path_usages = vec![];
    nodes.iter().for_each(|node| {
        add_path_usages_in_node(node, &mut path_usages);
    });

    path_usages
}

fn prepare_map_for_paths(
    nodes: &Vec<Node>,
    path_usages: Vec<(Node, u32)>,
) -> Vec<(Node, String, bool)> {
    let used_ids = find_ids_for_subtree(nodes);
    let id_generator = IdGenerator::new(used_ids);
    path_usages
        .into_iter()
        .filter_map(|(node, count)| (count > 1).then_some(node))
        .zip(id_generator)
        .map(|(path, id)| (path, id, false))
        .collect()
}

fn replace_id_of_node(node: Node, new_id: &str, id_map: &mut BTreeMap<String, String>) -> Node {
    if let Node::RegularNode {
        node_type,
        namespace,
        mut attributes,
        children,
    } = node
    {
        if let Some(id_attribute) = find_attribute_mut(&mut attributes, ID_NAME) {
            let old_id = id_attribute.clone();
            id_map.insert(old_id, new_id.into());
            *id_attribute = new_id.into();
        } else {
            attributes.push(OwnedAttribute {
                name: OwnedName::local(ID_NAME),
                value: new_id.into(),
            });
        }

        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        }
    } else {
        node
    }
}

fn merge_paths_for_node(
    node: Node,
    paths: &mut Vec<(Node, String, bool)>,
    id_map: &mut BTreeMap<String, String>,
) -> Node {
    match node {
        node @ Node::RegularNode {
            node_type: RegularNodeType::Path,
            ..
        } => {
            if let Some((_, id, first_found)) = paths
                .iter_mut()
                .find(|(path, _, _)| are_equal_paths(path, &node))
            {
                let new_node = replace_id_of_node(node, id, id_map);
                if !*first_found {
                    *first_found = true;
                    new_node
                } else {
                    Node::RegularNode {
                        node_type: RegularNodeType::Use,
                        namespace: NodeNamespace::empty(),
                        attributes: vec![OwnedAttribute::new(
                            OwnedName::local(HREF_NAME),
                            format!("#{}", id),
                        )],
                        children: vec![],
                    }
                }
            } else {
                node
            }
        }
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: children.map_to_vec(|child| merge_paths_for_node(child, paths, id_map)),
        },
        other => other,
    }
}

pub fn convert_paths_to_uses(nodes: Vec<Node>) -> Vec<Node> {
    let path_usages = find_path_usages(&nodes);
    let mut paths_map = prepare_map_for_paths(&nodes, path_usages);
    let mut id_map = BTreeMap::new();

    let new_nodes =
        nodes.map_to_vec(|node| merge_paths_for_node(node, &mut paths_map, &mut id_map));

    replace_ids(new_nodes, &id_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_convert_paths_to_uses,
        convert_paths_to_uses,
        r##"<svg xmlns="http://www.w3.org/2000/svg">
        <path id="abcd1" d="M150 5 L75 200 L225 200 Z"/>
        <g>
            <path id="abcd2" d="M150 5 L75 200 L225 200 Z"/>
            <path id="abcd3" d="M150 5 L75 200 L225 400 Z"/>
            <path fill="#012345" d="M150 5 L75 200 L225 200 Z"/>
            <path fill="#012345" d="M150 5 L75 200 L225 200 Z"/>
        </g>
        <path id="abcd4" d="M150 5 L75 200 L225 200 Z"/>
        </svg>"##,
        r##"<svg xmlns="http://www.w3.org/2000/svg">
        <path id="g" d="M150 5 L75 200 L225 200 Z"/>
        <g>
            <use href="#g"/>
            <path id="abcd3" d="M150 5 L75 200 L225 400 Z"/>
            <path fill="#012345" d="M150 5 L75 200 L225 200 Z" id="h"/>
            <use href="#h"/>
        </g>
        <use href="#g"/>
        </svg>"##
    );

    test_optimize!(
        test_convert_paths_to_uses_with_used_ids,
        convert_paths_to_uses,
        r##"<svg xmlns="http://www.w3.org/2000/svg">
        <path id="abcd1" d="M150 5 L75 200 L225 200 Z"/>
        <path id="abcd2" d="M150 5 L75 200 L225 200 Z"/>
        <use href="#abcd1"/>
        <use href="#abcd2"/>
        </svg>"##,
        r##"<svg xmlns="http://www.w3.org/2000/svg">
        <path id="g" d="M150 5 L75 200 L225 200 Z"/>
        <use href="#g"/>
        <use href="#g"/>
        <use href="#g"/>
        </svg>"##
    );
}
