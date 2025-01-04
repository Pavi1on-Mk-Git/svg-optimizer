use xml::attribute::OwnedAttribute;

use crate::node::{ChildlessNodeType, Node, RegularNodeType};

pub fn remove_comments(nodes: Vec<Node>) -> Vec<Node> {
    nodes
        .into_iter()
        .filter(|node| {
            !matches!(
                node,
                Node::ChildlessNode {
                    node_type: ChildlessNodeType::Comment(_)
                }
            )
        })
        .map(|node| match node {
            Node::RegularNode {
                node_type,
                attributes,
                children,
            } => Node::RegularNode {
                node_type,
                attributes,
                children: remove_comments(children),
            },
            childless_node => childless_node,
        })
        .collect()
}

fn merge_attributes(
    parent: Vec<OwnedAttribute>,
    child: Vec<OwnedAttribute>,
) -> Vec<OwnedAttribute> {
    let mut result = child;
    result.extend(parent);
    result.dedup_by(|fst, snd| fst.name == snd.name);
    result
}

pub fn remove_useless_groups(nodes: Vec<Node>) -> Vec<Node> {
    let nodes: Vec<Node> = nodes
        .into_iter()
        .map(|node| match node {
            Node::RegularNode {
                node_type: RegularNodeType::Group,
                attributes: parent_attr,
                children,
            } => {
                let mut new_children = remove_useless_groups(children);

                if new_children.len() > 1 {
                    return Node::RegularNode {
                        node_type: RegularNodeType::Group,
                        attributes: parent_attr,
                        children: new_children,
                    };
                }

                let last = new_children.pop();

                if let Some(node) = last {
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
                } else {
                    Node::RegularNode {
                        node_type: RegularNodeType::Group,
                        attributes: parent_attr,
                        children: vec![],
                    }
                }
            }
            Node::RegularNode {
                node_type,
                attributes,
                children,
            } => Node::RegularNode {
                node_type,
                attributes,
                children: remove_useless_groups(children),
            },
            other => other,
        })
        .collect();

    nodes
        .into_iter()
        .filter(|node| {
            if let Node::RegularNode {
                node_type: RegularNodeType::Group,
                attributes: _,
                children,
            } = node
            {
                !children.is_empty()
            } else {
                true
            }
        })
        .collect()
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
