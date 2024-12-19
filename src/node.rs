use std::fmt;
use xml::attribute::OwnedAttribute;
use xml::common::XmlVersion;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::XmlEvent;

pub enum Node {
    RegularNode {
        node_type: RegularNodeType,
        attributes: Vec<OwnedAttribute>,
        children: Vec<Node>,
    },
    ChildlessNode {
        node_type: ChildlessNodeType,
    },
}

impl RegularNodeType {
    fn tags(self, attributes: Vec<OwnedAttribute>) -> (XmlEvent, XmlEvent) {
        let name: OwnedName = self.into();
        (
            XmlEvent::StartElement {
                name: name.clone(),
                attributes,
                namespace: Namespace::empty(),
            },
            XmlEvent::EndElement { name },
        )
    }
}

macro_rules! conversions {
    ($([$node_type:ident, $name:literal]),*) => {

        #[derive(Debug)]
        pub enum RegularNodeType {
            $($node_type,)*
            Unknown,
        }

        impl From<OwnedName> for RegularNodeType {
            fn from(value: OwnedName) -> Self {
                match value.local_name.as_str() {
                    $($name => Self::$node_type,)*
                    _ => Self::Unknown,
                }
            }
        }

        impl fmt::Display for RegularNodeType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
                let name = match self {
                    $(RegularNodeType::$node_type => $name,)*
                    RegularNodeType::Unknown => ""
                };
                write!(f, "{}", name)
            }
        }
    };
}

impl From<RegularNodeType> for OwnedName {
    fn from(value: RegularNodeType) -> Self {
        OwnedName {
            local_name: value.to_string(),
            namespace: None,
            prefix: None,
        }
    }
}

conversions!([Svg, "svg"], [Rectangle, "rect"]);

pub enum ChildlessNodeType {
    Document(XmlVersion, String, Option<bool>),
    ProcessingInstruction(String, Option<String>),
    Comment(String),
    Text(String),
}

impl ChildlessNodeType {
    fn tag(self) -> XmlEvent {
        match self {
            Self::Document(version, encoding, standalone) => XmlEvent::StartDocument {
                version,
                encoding,
                standalone,
            },
            Self::ProcessingInstruction(name, data) => {
                XmlEvent::ProcessingInstruction { name, data }
            }
            Self::Comment(text) => XmlEvent::Comment(text),
            Self::Text(text) => XmlEvent::Characters(text),
        }
    }
}

impl IntoIterator for Node {
    type Item = XmlEvent;
    type IntoIter = NodeIter;
    fn into_iter(self) -> NodeIter {
        match self {
            Node::RegularNode {
                node_type,
                attributes,
                children,
            } => {
                let (start_tag, end_tag) = node_type.tags(attributes);
                NodeIter::RegularNodeIter {
                    start_tag: Some(start_tag),
                    end_tag: Some(end_tag),
                    child_iter: None,
                    children_iter: children.into_iter(),
                }
            }
            Node::ChildlessNode { node_type } => NodeIter::ChildlessNodeIter {
                tag: Some(node_type.tag()),
            },
        }
    }
}

pub enum NodeIter {
    RegularNodeIter {
        start_tag: Option<XmlEvent>,
        end_tag: Option<XmlEvent>,
        child_iter: Option<Box<NodeIter>>,
        children_iter: <Vec<Node> as IntoIterator>::IntoIter,
    },
    ChildlessNodeIter {
        tag: Option<XmlEvent>,
    },
}

impl Iterator for NodeIter {
    type Item = XmlEvent;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::RegularNodeIter {
                start_tag,
                end_tag,
                child_iter,
                children_iter,
            } => {
                if let Some(start_tag) = start_tag.take() {
                    Some(start_tag)
                } else if let Some(iter) = child_iter {
                    if let Some(element) = iter.next() {
                        Some(element)
                    } else {
                        *child_iter = None;
                        self.next()
                    }
                } else if let Some(child) = children_iter.next() {
                    *child_iter = Some(Box::new(child.into_iter()));
                    self.next()
                } else {
                    end_tag.take()
                }
            }
            Self::ChildlessNodeIter { tag } => tag.take(),
        }
    }
}
