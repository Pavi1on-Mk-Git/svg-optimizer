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
}
