use crate::node::{ChildlessNodeType, Node};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ParserError;
    use crate::parser::Parser;
    use xml::EventWriter;

    macro_rules! test_optimize {
        ($fn_name:ident, $tested_fn:ident, $test_str:literal, $result:literal) => {
            #[test]
            fn test_remove_comments() -> Result<(), ParserError> {
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

                assert_eq!($result, result);

                Ok(())
            }
        };
    }

    test_optimize!(
        test_remove_comments,
        remove_comments,
        "\
        <!-- comment -->\
        <svg width=\"320\" height=\"130\" xmlns=\"http://www.w3.org/2000/svg\">\
        <!-- comment -->\
        </svg>\
        <!-- comment -->\
        ",
        "\
        <?xml version=\"1.0\" encoding=\"UTF-8\"?><svg width=\"320\" height=\"130\" />\
        "
    );
}
