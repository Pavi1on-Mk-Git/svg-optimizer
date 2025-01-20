use crate::node::{ChildlessNodeType::*, NodeNamespace};
use crate::node::{Node, Node::*};
use anyhow::Result;
use std::io::Read;
use xml::name::OwnedName;
use xml::{
    attribute::OwnedAttribute,
    namespace::Namespace,
    reader::{ParserConfig2, XmlEvent},
    EventReader,
};

/// Parses input stream of events provided by xml library into the internal node tree format.
///
/// Because of library limitations, xml and DOCTYPE declarations are skipped.
/// Whitespace between attributes, as well as their order, are lost as well.
pub struct Parser<R: Read> {
    source: EventReader<R>,
    curr_event: Option<XmlEvent>,
}

impl<R: Read> Parser<R> {
    pub fn new(source: R) -> Result<Self> {
        let mut parser = Parser {
            source: ParserConfig2::new()
                .ignore_comments(false)
                .whitespace_to_characters(true)
                .ignore_root_level_whitespace(false)
                .create_reader(source),
            curr_event: None,
        };
        parser.next_event()?;
        Ok(parser)
    }

    fn next_event(&mut self) -> Result<()> {
        self.curr_event = match self.source.next()? {
            XmlEvent::EndDocument => None,
            event => Some(event),
        };
        Ok(())
    }

    pub fn parse_document(&mut self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();

        while self.curr_event.is_some() {
            if let Some(node) = self.parse_node()? {
                nodes.push(node);
            }

            self.next_event()?;
        }
        Ok(nodes)
    }

    fn parse_node(&mut self) -> Result<Option<Node>> {
        if let Some(XmlEvent::EndElement { .. }) = self.curr_event {
            return Ok(None);
        }

        let node = match self.curr_event.take() {
            Some(XmlEvent::StartDocument { .. }) => None,
            Some(XmlEvent::ProcessingInstruction { name, data }) => Some(ChildlessNode {
                node_type: ProcessingInstruction(name, data),
            }),
            Some(XmlEvent::CData(text)) => Some(ChildlessNode {
                node_type: Text(text, true),
            }),
            Some(XmlEvent::Comment(text)) => Some(ChildlessNode {
                node_type: Comment(text),
            }),
            Some(XmlEvent::Characters(text)) => Some(ChildlessNode {
                node_type: Text(text, false),
            }),
            Some(XmlEvent::StartElement {
                attributes,
                namespace,
                ..
            }) => Some(self.parse_regular_node(attributes, namespace)?),
            _ => unreachable!(),
        };
        Ok(node)
    }

    fn parse_regular_node(
        &mut self,
        attributes: Vec<OwnedAttribute>,
        element_namespace: Namespace,
    ) -> Result<Node> {
        let mut children = Vec::new();
        loop {
            self.next_event()?;

            if let Some(node) = self.parse_node()? {
                children.push(node);
            } else {
                return Ok(self.assemble_regular_node(attributes, element_namespace, children));
            }
        }
    }

    fn assemble_regular_node(
        &mut self,
        attributes: Vec<OwnedAttribute>,
        element_namespace: Namespace,
        children: Vec<Node>,
    ) -> Node {
        if let Some(XmlEvent::EndElement {
            name:
                OwnedName {
                    local_name,
                    namespace,
                    prefix,
                },
        }) = self.curr_event.take()
        {
            RegularNode {
                node_type: local_name.into(),
                namespace: NodeNamespace {
                    parent_namespace: namespace,
                    prefix,
                    element_namespace,
                },
                attributes,
                children,
            }
        } else {
            unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::RegularNodeType;

    #[test]
    fn test_parse_tag() -> Result<()> {
        let test_string =
            r#"<svg width="320" height="130" xmlns="http://www.w3.org/2000/svg"></svg>"#;

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 1);

        Ok(())
    }

    #[test]
    fn test_parse_nested_svg_tag() -> Result<()> {
        let test_string = r#"<svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
                             <svg width="320" height="130"><circle cx="50" cy="50" r="5"/></svg>
                             </svg>"#;

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 1);
        let only_node = nodes.into_iter().next().unwrap();
        match only_node {
            RegularNode {
                node_type,
                namespace,
                attributes,
                children,
            } => {
                let mut element_namespace = Namespace::empty();
                element_namespace.put("", "http://www.w3.org/2000/svg");
                element_namespace.put("xml", "http://www.w3.org/XML/1998/namespace");
                element_namespace.put("xmlns", "http://www.w3.org/2000/xmlns/");
                assert_eq!(node_type, RegularNodeType::Svg);
                assert_eq!(namespace.element_namespace, element_namespace);
                assert_eq!(attributes.len(), 2);
                assert_eq!(children.len(), 3); // 2 whitespace children
            }
            _ => {
                panic!();
            }
        }

        Ok(())
    }

    #[test]
    fn test_parse_oneline_tag() -> Result<()> {
        let test_string = r#"<svg width="320" height="130" xmlns="http://www.w3.org/2000/svg"/>"#;

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 1);

        Ok(())
    }

    #[test]
    fn test_no_start_tag() -> Result<()> {
        let test_string = r#"
            </svg>
            "#;

        let mut parser = Parser::new(test_string.as_bytes())?;

        assert!(parser.parse_document().is_err());

        Ok(())
    }

    #[test]
    fn test_no_end_tag() -> Result<()> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
            "#;

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document();

        assert!(nodes.is_err());
        Ok(())
    }

    #[test]
    fn test_badly_nested_tags() -> Result<()> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
            <circle cx="100" cy="50" r="50">
            <rect x="10" y="10" width="100" height="100">
            </circle>
            </rect>
            </svg>
            "#;

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document();

        assert!(nodes.is_err());
        Ok(())
    }

    #[test]
    fn test_parse_non_tag() -> Result<()> {
        let test_string = r#"
            <!--Generator: Adobe Illustrator 15.1.0, SVG Export Plug-In .SVG Version: 6.00 Build 0)-->
            <!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
            <svg/>
            "#.trim();

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 4); // 2 whitespace children

        Ok(())
    }
}
