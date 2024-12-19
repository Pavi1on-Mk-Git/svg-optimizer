use crate::errors::ParserError;
use crate::node::ChildlessNodeType::*;
use crate::node::Node;
use crate::node::Node::*;
use std::io::Read;
use xml::attribute::OwnedAttribute;
use xml::reader::ParserConfig2;
use xml::reader::XmlEvent;
use xml::EventReader;

/// Parses input stream of events provided by svg library into the output tree format of the svg library.
/// Currently only supports tag &lt;svg&gt;.
pub struct Parser<R: Read> {
    source: EventReader<R>,
    curr_event: Option<XmlEvent>,
}

type Result<T> = std::result::Result<T, ParserError>;

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
            Some(XmlEvent::StartDocument {
                version,
                encoding,
                standalone,
            }) => ChildlessNode {
                node_type: Document(version, encoding, standalone),
            },
            Some(XmlEvent::ProcessingInstruction { name, data }) => ChildlessNode {
                node_type: ProcessingInstruction(name, data),
            },
            Some(XmlEvent::Comment(text)) => ChildlessNode {
                node_type: Comment(text),
            },
            Some(XmlEvent::Characters(text)) => ChildlessNode {
                node_type: Text(text),
            },
            Some(XmlEvent::StartElement { attributes, .. }) => {
                self.parse_regular_node(attributes)?
            }
            _ => unreachable!(),
        };
        Ok(Some(node))
    }

    fn parse_regular_node(&mut self, attributes: Vec<OwnedAttribute>) -> Result<Node> {
        let mut children = Vec::new();
        loop {
            self.next_event()?;

            if let Some(node) = self.parse_node()? {
                children.push(node);
            } else {
                return Ok(self.assemble_regular_node(attributes, children));
            }
        }
    }

    fn assemble_regular_node(
        &mut self,
        attributes: Vec<OwnedAttribute>,
        children: Vec<Node>,
    ) -> Node {
        if let Some(XmlEvent::EndElement { name }) = self.curr_event.take() {
            RegularNode {
                node_type: name.into(),
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

        assert_eq!(nodes.len(), 2);

        Ok(())
    }

    #[test]
    fn test_parse_oneline_tag() -> Result<()> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg"/>
            "#;

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 2);

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
    fn test_parse_non_tag() -> Result<()> {
        let test_string = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <!--Generator: Adobe Illustrator 15.1.0, SVG Export Plug-In .SVG Version: 6.00 Build 0)-->
            <!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
            <svg/>
            "#.trim();

        let mut parser = Parser::new(test_string.as_bytes())?;

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 3);

        Ok(())
    }
}
