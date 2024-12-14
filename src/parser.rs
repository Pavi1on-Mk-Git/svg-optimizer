use crate::errors::ParserError;
use std::cell::RefCell;
use svg::node::element;
use svg::node::element::tag;
use svg::node::element::tag::Type;
use svg::node::{Blob, Comment};
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
    ($fn_name:ident, $tag:ident, $element:ident, $($parse_fn:ident),* $(once: $parse_once_fn:ident)?) => {
        fn $fn_name(&mut self) -> NodeResult {
            match &self.curr_event {
                Some(Event::Tag(tag::$tag, Type::Start, attr)) => {
                    let mut element = element::$element::new();
                    let once_node: RefCell<Option<Box<dyn Node>>> = RefCell::new(None);

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

                        $(
                            once_node.replace(once_node.take().or(self.$parse_once_fn()?));
                        )?

                        match &self.curr_event {
                            Some(Event::Tag(tag::$tag, Type::End, _)) => {
                                if let Some(node) = once_node.take() {
                                    element = element.add(node);
                                }

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
                    let mut element = element::$element::new();
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

macro_rules! parse_character_data_element {
    ($fn_name:ident, $tag:ident, $element:ident) => {
        fn $fn_name(&mut self) -> NodeResult {
            match &self.curr_event {
                Some(Event::Tag(tag::$tag, Type::Start, attr)) => {
                    let mut element: Option<element::$element> = None;
                    let mut nodes = Vec::new();
                    let attributes = attr.clone();

                    loop {
                        self.next_event();

                        if let Some(node) = self.parse_non_tag()? {
                            nodes.push(node);
                        }

                        match &self.curr_event {
                            Some(Event::Tag(tag::$tag, Type::End, _)) => {
                                for node in nodes {
                                    element = element.map(|el| el.add(node));
                                }

                                return Ok(Some(Box::new(
                                    element.unwrap_or(element::$element::new("")),
                                )));
                            }
                            Some(Event::Text(text)) => {
                                element = Some(element::$element::new(*text));
                                for (key, val) in &attributes {
                                    element = element.map(|el| el.set(key, val.clone()));
                                }
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
                    let mut element = element::$element::new("");
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
                Event::Error(error) => Err(error.into()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    parse_element!(
        parse_svg,
        SVG,
        SVG,
        parse_animation_element,
        parse_basic_shape
    );

    parse_element_group!(
        parse_animation_element,
        parse_animate,
        parse_animate_motion,
        parse_animate_transform
    );

    parse_element_group!(
        parse_basic_shape,
        parse_circle,
        parse_ellipse,
        parse_line,
        parse_path,
        parse_polygon,
        parse_polyline,
        parse_rect
    );

    parse_element_group!(parse_descriptive_element,);

    parse_element_group!(parse_paint_server_element,);

    parse_element!(
        parse_animate,
        Animate,
        Animate,
        parse_descriptive_element,
        parse_script
    );

    parse_element!(
        parse_animate_motion,
        AnimateMotion,
        AnimateMotion,
        parse_descriptive_element,
        parse_script
        once: parse_mpath
    );

    parse_element!(
        parse_animate_transform,
        AnimateTransform,
        AnimateTransform,
        parse_descriptive_element,
        parse_script
    );

    parse_element!(
        parse_circle,
        Circle,
        Circle,
        parse_animation_element,
        parse_descriptive_element,
        parse_paint_server_element,
        parse_clip_path,
        parse_marker,
        parse_mask,
        parse_script,
        parse_style
    );

    parse_element!(parse_clip_path, ClipPath, ClipPath,);

    parse_element!(
        parse_ellipse,
        Ellipse,
        Ellipse,
        parse_animation_element,
        parse_descriptive_element,
        parse_paint_server_element,
        parse_clip_path,
        parse_marker,
        parse_mask,
        parse_script,
        parse_style
    );

    parse_element!(
        parse_line,
        Line,
        Line,
        parse_animation_element,
        parse_descriptive_element,
        parse_paint_server_element,
        parse_clip_path,
        parse_marker,
        parse_mask,
        parse_script,
        parse_style
    );

    parse_element!(parse_marker, Marker, Marker,);

    parse_element!(parse_mask, Mask, Mask,);

    parse_element!(
        parse_mpath,
        MotionPath,
        MotionPath,
        parse_descriptive_element,
        parse_script
    );

    parse_element!(
        parse_path,
        Path,
        Path,
        parse_animation_element,
        parse_descriptive_element,
        parse_paint_server_element,
        parse_clip_path,
        parse_marker,
        parse_mask,
        parse_script,
        parse_style
    );

    parse_element!(
        parse_polygon,
        Polygon,
        Polygon,
        parse_animation_element,
        parse_descriptive_element,
        parse_paint_server_element,
        parse_clip_path,
        parse_marker,
        parse_mask,
        parse_script,
        parse_style
    );

    parse_element!(
        parse_polyline,
        Polyline,
        Polyline,
        parse_animation_element,
        parse_descriptive_element,
        parse_paint_server_element,
        parse_clip_path,
        parse_marker,
        parse_mask,
        parse_script,
        parse_style
    );

    parse_element!(
        parse_rect,
        Rectangle,
        Rectangle,
        parse_animation_element,
        parse_descriptive_element,
        parse_paint_server_element,
        parse_clip_path,
        parse_marker,
        parse_mask,
        parse_script,
        parse_style
    );

    parse_character_data_element!(parse_script, Script, Script);

    parse_character_data_element!(parse_style, Style, Style);
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
