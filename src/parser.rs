use crate::errors::ParserError;
use svg::node::element::tag::Type;
use svg::node::element::*;
use svg::parser::Event;
use svg::Node;

/// Parses input stream of events provided by svg library into the output tree format of the svg library.
/// Currently only supports tag &lt;svg&gt;.
pub struct Parser<'a> {
    source: svg::Parser<'a>,
    curr_event: Option<Event<'a>>,
}

type NodeResult = Result<Option<Box<dyn Node>>, ParserError>;

macro_rules! add_parsed {
    ($document:ident, $($parse_call:expr),*) => {
        $(
            if let Some(node) = $parse_call? {
                $document = $document.add(node)
            }
        )*
    };
}

macro_rules! parse_element {
    ($fn_name:ident, $tag:ident, $element:ty, $($parse_fn:ident),*) => {
        fn $fn_name(&mut self) -> NodeResult {
            match &self.curr_event {
                Some(Event::Tag(tag::$tag, Type::Start, attr)) => {
                    let mut element = <$element>::new();
                    for (key, val) in attr {
                        element = element.set(key, val.clone());
                    }

                    loop {
                        self.next_event();

                        $(add_parsed!(element, self.$parse_fn());)*

                        match &self.curr_event {
                            Some(Event::Tag(tag::$tag, Type::End, _)) => {
                                return Ok(Some(Box::new(element)))
                            }
                            None => {
                                return Err(ParserError::MissingEndTag {
                                    tag_type: tag::$tag.into(),
                                })
                            }
                            _ => {}
                        }
                    }
                }
                Some(Event::Tag(tag::$tag, Type::Empty, attr)) => {
                    let mut element = <$element>::new();
                    for (key, val) in attr {
                        element = element.set(key, val.clone());
                    }
                    Ok(Some(Box::new(element)))
                }
                _ => Ok(None),
            }
        }
    };
}

macro_rules! parse_element_group {
    ($fn_name:ident, $($parse_fn:ident),*) => {
        fn $fn_name(&mut self) -> NodeResult {
            $(
                if let Some(node) = self.$parse_fn()? {
                    return Ok(Some(node));
                }
            )*
            Ok(None)
        }
    };
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

    parse_element_group!(parse_animation_element,);

    parse_element_group!(parse_descriptive_element,);

    parse_element!(
        parse_element,
        Circle,
        Circle,
        parse_animation_element,
        parse_descriptive_element
    );

    parse_element!(
        parse_ellipse,
        Ellipse,
        Ellipse,
        parse_animation_element,
        parse_descriptive_element
    );

    parse_element_group!(parse_basic_shape, parse_element, parse_ellipse);

    parse_element!(parse_svg, SVG, SVG, parse_basic_shape);

    pub fn parse_document(&mut self) -> Result<(Box<dyn Node>, Vec<String>), ParserError> {
        let mut strings_to_keep: Vec<String> = Vec::new();

        loop {
            if let Some(event) = &self.curr_event {
                match event {
                    Event::Error(error) => return Err(error.into()),
                    Event::Comment(text) => strings_to_keep.push((*text).into()),
                    Event::Declaration(text) => strings_to_keep.push((*text).into()),
                    Event::Instruction(text) => strings_to_keep.push((*text).into()),
                    Event::Text(_) => return Err(ParserError::UnexpectedText),
                    Event::Tag(..) => {
                        if let Some(svg) = self.parse_svg()? {
                            return Ok((svg, strings_to_keep));
                        } else {
                            return Err(ParserError::MissingSVGStart);
                        }
                    }
                }
            } else {
                return Err(ParserError::MissingSVGStart);
            }
            self.next_event();
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
            </svg>
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let (document, strings) = parser.parse_document()?;

        assert!(strings.is_empty());

        let attrs = document.get_attributes().unwrap();

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
    fn test_parse_oneline_svg() -> Result<(), ParserError> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg"/>
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let (document, strings) = parser.parse_document()?;

        assert!(strings.is_empty());

        let attrs = document.get_attributes().unwrap();

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
    fn test_svg_no_end_tag() {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let document = parser.parse_document();

        assert!(document.is_err());
    }

    #[test]
    fn test_parse_non_svg() -> Result<(), ParserError> {
        let test_string = r#"
            <circle cx="50" cy="50" r="50">
            </circle>
            "#;

        let source = svg::Parser::new(test_string);
        let mut parser = Parser::new(source);

        let element = parser.parse_element()?.unwrap();

        let attrs = element.get_attributes().unwrap();

        assert_eq!(attrs.len(), 3);

        assert!(attrs.contains_key("cx"));
        assert!(attrs.contains_key("cy"));
        assert!(attrs.contains_key("r"));

        assert_eq!(*attrs.get("cx").unwrap(), "50");
        assert_eq!(*attrs.get("cy").unwrap(), "50");
        assert_eq!(*attrs.get("r").unwrap(), "50");

        Ok(())
    }

    #[test]
    fn test_parse_oneline_non_svg() -> Result<(), ParserError> {
        let test_string = r#"
            <circle cx="50" cy="50" r="50"/>
            "#;

        let source = svg::Parser::new(test_string);
        let mut parser = Parser::new(source);

        let element = parser.parse_element()?.unwrap();

        let attrs = element.get_attributes().unwrap();

        assert_eq!(attrs.len(), 3);

        assert!(attrs.contains_key("cx"));
        assert!(attrs.contains_key("cy"));
        assert!(attrs.contains_key("r"));

        assert_eq!(*attrs.get("cx").unwrap(), "50");
        assert_eq!(*attrs.get("cy").unwrap(), "50");
        assert_eq!(*attrs.get("r").unwrap(), "50");

        Ok(())
    }

    #[test]
    fn test_no_end_tag_no_svg() {
        let test_string = r#"
            <circle cx="50" cy="50" r="50">
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let element = parser.parse_element();

        assert!(element.is_err());
    }
}
