use super::EasyIter;
use crate::node::{Node, RegularNodeType};
use anyhow::Result;
use xml::attribute::OwnedAttribute;

fn remove_useless_groups_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Group,
            attributes: parent_attr,
            children,
        } => {
            let mut new_children: Vec<Node> = children.filter_map(remove_useless_groups_from_node);

            match new_children.len() {
                0 => None,
                1 => Some(create_new_group(new_children.remove(0), parent_attr)),
                _ => Some(Node::RegularNode {
                    node_type: RegularNodeType::Group,
                    attributes: parent_attr,
                    children: new_children,
                }),
            }
        }
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            attributes,
            children: children.filter_map(remove_useless_groups_from_node),
        }),
        other => Some(other),
    }
}

fn create_new_group(only_child: Node, group_attributes: Vec<OwnedAttribute>) -> Node {
    if let Node::RegularNode {
        node_type,
        attributes,
        children,
    } = only_child
    {
        Node::RegularNode {
            node_type,
            attributes: merge_attributes(group_attributes, attributes),
            children,
        }
    } else {
        Node::RegularNode {
            node_type: RegularNodeType::Group,
            attributes: group_attributes,
            children: vec![only_child],
        }
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

pub fn remove_useless_groups(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.filter_map(remove_useless_groups_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::test::test_optimize;
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
}
