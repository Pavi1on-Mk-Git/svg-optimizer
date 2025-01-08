use xml::attribute::OwnedAttribute;

use crate::node::{ChildlessNodeType, Node, RegularNodeType};

fn apply_to_nodes<I, F>(nodes: I, func: F) -> Vec<Node>
where
    I: IntoIterator<Item = Node>,
    F: Fn(Node) -> Option<Node>,
{
    nodes.into_iter().filter_map(func).collect()
}

fn remove_comments_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            attributes,
            children: remove_comments(children),
        }),
        Node::ChildlessNode {
            node_type: ChildlessNodeType::Comment(_),
        } => None,
        childless_node => Some(childless_node),
    }
}

pub fn remove_comments<I: IntoIterator<Item = Node>>(nodes: I) -> Vec<Node> {
    apply_to_nodes(nodes, remove_comments_from_node)
}

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

pub fn remove_useless_groups<I: IntoIterator<Item = Node>>(nodes: I) -> Vec<Node> {
    apply_to_nodes(nodes, remove_useless_groups_from_node)
}

fn ellipsis_to_circles_from_node(node: Node) -> Option<Node> {
    Some(match node {
        Node::RegularNode {
            node_type: RegularNodeType::Ellipse,
            attributes,
            children,
        } => {
            let children = ellipsis_to_circles(children);

            let (node_type, attributes) = match circle_attributes(attributes) {
                Ok(attributes) => (RegularNodeType::Circle, attributes),
                Err(attributes) => (RegularNodeType::Ellipse, attributes),
            };

            Node::RegularNode {
                node_type,
                attributes,
                children,
            }
        }
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            attributes,
            children: ellipsis_to_circles(children),
        },
        childless_node => childless_node,
    })
}

fn circle_attributes(
    attributes: Vec<OwnedAttribute>,
) -> Result<Vec<OwnedAttribute>, Vec<OwnedAttribute>> {
    let rx_name = "rx";
    let ry_name = "ry";

    let rx = attributes
        .iter()
        .find(|attr| attr.name.local_name == rx_name);
    let ry = attributes
        .iter()
        .find(|attr| attr.name.local_name == ry_name);

    match (rx, ry) {
        (Some(rx), Some(ry)) if rx.value == ry.value => {
            let mut r_name = rx.name.clone();
            r_name.local_name = "r".into();

            let r_val = rx.value.clone();

            let mut attributes: Vec<OwnedAttribute> = attributes
                .into_iter()
                .filter(|attr| {
                    let name = &attr.name.local_name;
                    name != rx_name && name != ry_name
                })
                .collect();
            attributes.push(OwnedAttribute {
                name: r_name,
                value: r_val,
            });
            Ok(attributes)
        }
        _ => Err(attributes),
    }
}

pub fn ellipsis_to_circles<I: IntoIterator<Item = Node>>(nodes: I) -> Vec<Node> {
    apply_to_nodes(nodes, ellipsis_to_circles_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ParserError;
    use crate::parser::Parser;
    use xml::EventWriter;

    macro_rules! test_optimize {
        ($test_name:ident, $tested_fn:ident, $test_str:literal, $result:literal) => {
            #[test]
            fn $test_name() -> Result<(), ParserError> {
                let test_string = $test_str;

                let mut parser = Parser::new(test_string.as_bytes())?;

                let nodes = parser.parse_document()?;

                let nodes = $tested_fn(nodes);

                let buffer = Vec::new();
                let mut writer = EventWriter::new(buffer);

                nodes.into_iter().try_for_each(|node| {
                    node.into_iter()
                        .try_for_each(|event| writer.write(event.as_writer_event().unwrap()))
                })?;

                let result = String::from_utf8(writer.into_inner()).unwrap();

                assert_eq!(result, $result);

                Ok(())
            }
        };
    }

    test_optimize!(
        test_remove_comments,
        remove_comments,
        "\
        <!-- comment -->\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <!-- comment -->\
        </svg>\
        <!-- comment -->\
        ",
        "\
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>\
        <svg xmlns=\"http://www.w3.org/2000/svg\" />\
        "
    );

    test_optimize!(
        test_remove_useless_groups_removed,
        remove_useless_groups,
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <g fill=\"white\" stroke=\"green\" stroke-width=\"5\">\
        <circle cx=\"40\" cy=\"40\" r=\"25\" />\
        </g>\
        <g><g/></g>\
        </svg>\
        ",
        "\
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <circle cx=\"40\" cy=\"40\" r=\"25\" fill=\"white\" stroke=\"green\" stroke-width=\"5\" />\
        </svg>\
        "
    );

    test_optimize!(
        test_remove_useless_groups_not_removed,
        remove_useless_groups,
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <g fill=\"white\" stroke=\"green\" stroke-width=\"5\">\
        <circle cx=\"40\" cy=\"40\" r=\"25\" />\
        <circle cx=\"80\" cy=\"80\" r=\"25\" />\
        </g>\
        </svg>\
        ",
        "\
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <g fill=\"white\" stroke=\"green\" stroke-width=\"5\">\
        <circle cx=\"40\" cy=\"40\" r=\"25\" />\
        <circle cx=\"80\" cy=\"80\" r=\"25\" />\
        </g>\
        </svg>\
        "
    );

    test_optimize!(
        test_ellipsis_to_circles,
        ellipsis_to_circles,
        "\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <svg viewBox=\"0 0 200 100\" xmlns=\"http://www.w3.org/2000/svg\">\
        <ellipse cx=\"100\" cy=\"50\" rx=\"50\" ry=\"50\" />\
        </svg>
        </svg>\
        ",
        "\
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>\
        <svg xmlns=\"http://www.w3.org/2000/svg\">\
        <svg viewBox=\"0 0 200 100\" xmlns=\"http://www.w3.org/2000/svg\">\
        <circle cx=\"100\" cy=\"50\" r=\"50\" />\
        </svg>
        </svg>\
        "
    );
}
