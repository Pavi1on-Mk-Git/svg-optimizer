use super::common::{
    constants::ID_NAME,
    id_usage::{find_attribute, find_ids_for_subtree},
    iter::EasyIter,
};
use crate::node::{Node, NodeNamespace, RegularNodeType};
use xml::attribute::OwnedAttribute;

fn is_only_child_used(only_child: &Node, used_ids: &[String]) -> bool {
    if let Node::RegularNode { attributes, .. } = only_child {
        if let Some(id) = find_attribute(attributes, ID_NAME) {
            if used_ids.contains(id) {
                return true;
            }
        }
    }

    false
}

fn remove_useless_groups_from_node(node: Node, used_ids: &[String]) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Group,
            namespace: parent_namespace,
            attributes: parent_attr,
            children,
        } => {
            let mut new_children: Vec<Node> = children
                .filter_map_to_vec(|child| remove_useless_groups_from_node(child, used_ids));

            match new_children.len() {
                0 => None,
                1 if !is_only_child_used(&new_children[0], used_ids) => {
                    collapse_group(new_children.remove(0), parent_namespace, parent_attr)
                }
                _ => Some(Node::RegularNode {
                    node_type: RegularNodeType::Group,
                    namespace: parent_namespace,
                    attributes: parent_attr,
                    children: new_children,
                }),
            }
        }
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: children
                .filter_map_to_vec(|child| remove_useless_groups_from_node(child, used_ids)),
        }),
        other => Some(other),
    }
}

fn collapse_group(
    only_child: Node,
    group_namespace: NodeNamespace,
    group_attributes: Vec<OwnedAttribute>,
) -> Option<Node> {
    if let Node::RegularNode {
        node_type,
        namespace,
        attributes,
        children,
    } = only_child
    {
        Some(Node::RegularNode {
            node_type,
            namespace, //TODO Check if it needs merging of namespaces
            attributes: merge_attributes(group_attributes, attributes),
            children,
        })
    } else {
        Some(Node::RegularNode {
            node_type: RegularNodeType::Group,
            namespace: group_namespace,
            attributes: group_attributes,
            children: vec![only_child],
        })
    }
}

fn merge_attributes(
    parent: Vec<OwnedAttribute>,
    mut child: Vec<OwnedAttribute>,
) -> Vec<OwnedAttribute> {
    child.extend(parent);
    child.dedup_by(|fst, snd| fst.name == snd.name);
    child
}

pub fn remove_useless_groups(nodes: Vec<Node>) -> Vec<Node> {
    let used_ids = find_ids_for_subtree(&nodes);
    nodes.filter_map_to_vec(|node| remove_useless_groups_from_node(node, &used_ids))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_useless_groups_removed,
        remove_useless_groups,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <g fill="white" stroke="green" stroke-width="5"><circle cx="40" cy="40" r="25"/></g>
        <g><g/><g/></g></svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <circle cx="40" cy="40" r="25" fill="white" stroke="green" stroke-width="5"/>
        </svg>
        "#
    );

    test_optimize!(
        test_remove_useless_groups_not_removed,
        remove_useless_groups,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <g fill="white" stroke="green" stroke-width="5">
        <circle cx="40" cy="40" r="25"/>
        <circle cx="80" cy="80" r="25"/>
        </g>
        <g fill="white" stroke="green" stroke-width="5">
        some text
        </g>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <g fill="white" stroke="green" stroke-width="5">
        <circle cx="40" cy="40" r="25"/>
        <circle cx="80" cy="80" r="25"/>
        </g>
        <g fill="white" stroke="green" stroke-width="5">
        some text
        </g>
        </svg>
        "#
    );

    test_optimize!(
        test_remove_useless_groups_item_used_via_id,
        remove_useless_groups,
        r##"<svg viewBox="-40 0 150 100">
        <g transform="rotate(-10 50 100)"><path id="heart" d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"/></g>
        <use href="#heart" fill="none" stroke="red"/>
        </svg>
        "##,
        r##"<svg viewBox="-40 0 150 100">
        <g transform="rotate(-10 50 100)"><path id="heart" d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"/></g>
        <use href="#heart" fill="none" stroke="red"/>
        </svg>
        "##
    );
}
