use super::apply_to_nodes;
use crate::node::Node;
use crate::node::RegularNodeType;
use xml::attribute::OwnedAttribute;

fn remove_useless_groups_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Group,
            attributes: parent_attr,
            children,
        } => {
            let mut new_children = remove_useless_groups(children);

            if new_children.len() > 1 {
                return Some(Node::RegularNode {
                    node_type: RegularNodeType::Group,
                    attributes: parent_attr,
                    children: new_children,
                });
            }

            new_children
                .pop()
                .map(|node| merge_with_group(node, parent_attr, new_children))
        }
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            attributes,
            children: remove_useless_groups(children),
        }),
        other => Some(other),
    }
}

fn merge_with_group(
    node: Node,
    parent_attr: Vec<OwnedAttribute>,
    mut new_children: Vec<Node>,
) -> Node {
    if let Node::RegularNode {
        node_type,
        attributes: child_attr,
        children,
    } = node
    {
        Node::RegularNode {
            node_type,
            attributes: merge_attributes(parent_attr, child_attr),
            children,
        }
    } else {
        new_children.push(node);
        Node::RegularNode {
            node_type: RegularNodeType::Group,
            attributes: parent_attr,
            children: new_children,
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

pub fn remove_useless_groups(nodes: Vec<Node>) -> Vec<Node> {
    apply_to_nodes(nodes, remove_useless_groups_from_node)
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
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <g fill=\"white\" stroke=\"green\" stroke-width=\"5\">\
        <circle cx=\"40\" cy=\"40\" r=\"25\"/>\
        </g>\
        <g><g/></g>\
        </svg>\
        ",
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <circle cx=\"40\" cy=\"40\" r=\"25\" fill=\"white\" stroke=\"green\" stroke-width=\"5\"/>\
        </svg>\
        "
    );

    test_optimize!(
        test_remove_useless_groups_not_removed,
        remove_useless_groups,
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <g fill=\"white\" stroke=\"green\" stroke-width=\"5\">\
        <circle cx=\"40\" cy=\"40\" r=\"25\"/>\
        <circle cx=\"80\" cy=\"80\" r=\"25\"/>\
        </g>\
        </svg>\
        ",
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <g fill=\"white\" stroke=\"green\" stroke-width=\"5\">\
        <circle cx=\"40\" cy=\"40\" r=\"25\"/>\
        <circle cx=\"80\" cy=\"80\" r=\"25\"/>\
        </g>\
        </svg>\
        "
    );
}
