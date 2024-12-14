use crate::errors::ParserError;
use svg::node::element::tag::Type;
use svg::node::{element::*, Blob, Comment};
use svg::parser::Event;
use svg::Node;

/// Parses input stream of events provided by svg library into the output tree format of the svg library.
/// Currently only supports tag &lt;svg&gt;.
pub struct Parser<'a> {
    source: svg::Parser<'a>,
    curr_event: Option<Event<'a>>,
}

type NodeResult = Result<Option<Box<dyn Node>>, ParserError>;

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

                        if let Some(node) = self.parse_non_tag()? {
                            element = element.add(node);
                        }

                        $(
                            if let Some(node) = self.$parse_fn()? {
                                element = element.add(node)
                            }
                        )*

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
            if let Some(node) = self.parse_non_tag()? {
                return Ok(Some(node));
            }

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

    fn parse_non_tag(&mut self) -> NodeResult {
        if let Some(event) = &self.curr_event {
            match event {
                Event::Declaration(text) | Event::Instruction(text) => {
                    Ok(Some(Box::new(Blob::new(*text))))
                }
                Event::Comment(text) => Ok(Some(Box::new(Comment::new(
                    text.strip_prefix("<!--")
                        .and_then(|txt| txt.strip_suffix("-->"))
                        .unwrap()
                        .trim(),
                )))),
                Event::Text(text) => Ok(Some(Box::new(Text::new(*text)))),
                Event::Error(error) => Err(error.into()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    parse_element_group!(parse_animation_element,);

    parse_element_group!(parse_descriptive_element,);

    parse_element!(
        parse_circle,
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

    parse_element_group!(parse_basic_shape, parse_circle, parse_ellipse);

    parse_element!(parse_svg, SVG, SVG, parse_basic_shape);

    pub fn parse_document(&mut self) -> Result<Vec<Box<dyn Node>>, ParserError> {
        let mut nodes = Vec::new();

        while self.curr_event.is_some() {
            if let Some(node) = self.parse_non_tag()? {
                nodes.push(node);
            }

            if let Some(node) = self.parse_svg()? {
                nodes.push(node);
            }

            self.next_event();
        }
        Ok(nodes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag() -> Result<(), ParserError> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
            </svg>
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 1);

        let document = &nodes[0];

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
    fn test_parse_oneline_tag() -> Result<(), ParserError> {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg"/>
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 1);

        let document = &nodes[0];

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
    fn test_no_start_tag() -> Result<(), ParserError> {
        let test_string = r#"
            </svg>
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let nodes = parser.parse_document()?;

        assert!(nodes.is_empty());

        Ok(())
    }

    #[test]
    fn test_no_end_tag() {
        let test_string = r#"
            <svg width="320" height="130" xmlns="http://www.w3.org/2000/svg">
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let nodes = parser.parse_document();

        assert!(nodes.is_err());
    }

    #[test]
    fn test_parse_non_tag() -> Result<(), ParserError> {
        let test_string = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <!--Generator: Adobe Illustrator 15.1.0, SVG Export Plug-In .SVG Version: 6.00 Build 0)-->
            <!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
            "#;

        let source = svg::Parser::new(test_string);

        let mut parser = Parser::new(source);

        let nodes = parser.parse_document()?;

        assert_eq!(nodes.len(), 3);

        assert_eq!(
            nodes[0].to_string(),
            r#"<?xml version="1.0" encoding="utf-8"?>"#
        );
        assert_eq!(
            nodes[1].to_string(),
            r#"<!-- Generator: Adobe Illustrator 15.1.0, SVG Export Plug-In .SVG Version: 6.00 Build 0) -->"#
        );
        assert_eq!(
            nodes[2].to_string(),
            r#"<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">"#
        );

        Ok(())
    }
}
