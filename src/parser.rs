use crate::ParserError;
use svg::node::element::tag::{Type, SVG};
use svg::parser::Event;
use svg::Document;

/// Parses input stream of events provided by svg library into the output tree format of the svg library.
/// Currently only supports tag &lt;svg&gt;.
pub struct Parser<'a> {
    source: svg::Parser<'a>,
    curr_event: Option<Event<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: svg::Parser<'a>) -> Self {
        let mut parser = Parser {
            source,
            curr_event: None,
        };

        parser.next_event();
        parser
    }

    fn next_event(&mut self) {
        self.curr_event = self.source.next();
    }

    pub fn parse_document(&mut self) -> Result<Document, ParserError> {
        match &self.curr_event {
            Some(Event::Tag(SVG, Type::Start, attr)) => {
                let mut document = Document::new();
                for (key, val) in attr {
                    document = document.set(key, val.clone());
                }

                loop {
                    self.next_event();

                    match &self.curr_event {
                        Some(Event::Tag(SVG, Type::End, _)) => return Ok(document),
                        None => return Err(ParserError::new("No end tag")),
                        _ => {}
                    }
                }
            }
            _ => Err(ParserError::new("No start tag")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_svg() -> Result<(), ParserError> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
            <rect width="300" height="100" x="10" y="10" style="fill:rgb(0,0,255);stroke-width:3;stroke:red" />
            </svg>
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let document = parser.parse_document()?;

        let attrs = document.get_attributes();

        assert_eq!(attrs.len(), 3);

        assert!(attrs.contains_key("width"));
        assert!(attrs.contains_key("height"));
        assert!(attrs.contains_key("xmlns"));

        assert_eq!(*attrs.get("width").unwrap(), "320");
        assert_eq!(*attrs.get("height").unwrap(), "130");
        assert_eq!(*attrs.get("xmlns").unwrap(), "http://www.w3.org/2000/svg");

        Ok(())
    }

    #[test]
    fn test_no_start_tag() {
        let test_string = r#"
            </svg>
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let document = parser.parse_document();

        assert!(document.is_err());
    }

    #[test]
    fn test_no_end_tag() {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let document = parser.parse_document();

        assert!(document.is_err());
    }
}
