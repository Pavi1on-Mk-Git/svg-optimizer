use crate::node::ChildlessNodeType::*;
use crate::node::Node;
use crate::node::Node::*;
use crate::node::RegularNodeType;
use anyhow::Result;
use std::io::Read;
use xml::attribute::OwnedAttribute;
use xml::namespace::Namespace;
use xml::reader::ParserConfig2;
use xml::reader::XmlEvent;
use xml::EventReader;

/// Parses input stream of events provided by svg library into the output tree format of the svg library.
/// Currently only supports tag &lt;svg&gt;.
pub struct Parser<R: Read> {
    source: EventReader<R>,
    curr_event: Option<XmlEvent>,
}

impl<R: Read> Parser<R> {
    pub fn new(source: R) -> Result<Self> {
        let mut parser = Parser {
            source: ParserConfig2::new()
                .ignore_comments(false)
                .cdata_to_characters(true)
                .whitespace_to_characters(true)
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
            Some(XmlEvent::Comment(text)) => Some(ChildlessNode {
                node_type: Comment(text),
            }),
            Some(XmlEvent::Characters(text)) => Some(ChildlessNode {
                node_type: Text(text),
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
        namespace: Namespace,
    ) -> Result<Node> {
        let mut children = Vec::new();
        loop {
            self.next_event()?;

            if let Some(node) = self.parse_node()? {
                children.push(node);
            } else {
                return Ok(self.assemble_regular_node(attributes, namespace, children));
            }
        }
    }

    fn assemble_regular_node(
        &mut self,
        attributes: Vec<OwnedAttribute>,
        namespace: Namespace,
        children: Vec<Node>,
    ) -> Node {
        if let Some(XmlEvent::EndElement { name }) = self.curr_event.take() {
            let node_type = if name.local_name == "svg" {
                RegularNodeType::Svg(namespace.get("").map(|s| s.into()))
            } else {
                name.into()
            };
            RegularNode {
                node_type,
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

    #[test]
    fn test_parse_tag() -> Result<()> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
            </svg>
            "#;

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 1);

        Ok(())
    }

    #[test]
    fn test_parse_oneline_tag() -> Result<()> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg"/>
            "#;

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
            <circle cx=\"100\" cy=\"50\" r=\"50\">
            <rect x=\"10\" y=\"10\" width=\"100\" height=\"100\">\
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

        assert_eq!(nodes.len(), 2);

        Ok(())
    }
}
