use std::fmt;
use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::XmlEvent;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeNamespace {
    pub parent_namespace: Option<String>,
    pub prefix: Option<String>,
    pub element_namespace: Namespace,
}

impl NodeNamespace {
    pub fn empty() -> Self {
        Self {
            parent_namespace: None,
            prefix: None,
            element_namespace: Namespace::empty(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    RegularNode {
        node_type: RegularNodeType,
        namespace: NodeNamespace,
        attributes: Vec<OwnedAttribute>,
        children: Vec<Node>,
    },
    ChildlessNode {
        node_type: ChildlessNodeType,
    },
}

impl RegularNodeType {
    fn tags(
        self,
        namespace: NodeNamespace,
        attributes: Vec<OwnedAttribute>,
    ) -> (XmlEvent, XmlEvent) {
        let name = OwnedName {
            local_name: self.to_string(),
            namespace: namespace.parent_namespace,
            prefix: namespace.prefix,
        };
        (
            XmlEvent::StartElement {
                name: name.clone(),
                attributes,
                namespace: namespace.element_namespace,
            },
            XmlEvent::EndElement { name },
        )
    }
}

macro_rules! conversions {
    ($([$node_type:ident, $name:literal]),*) => {

        #[derive(Debug, PartialEq, Eq, Clone)]
        pub enum RegularNodeType {
            Unknown(String),
            $($node_type,)*
        }

        impl From<String> for RegularNodeType {
            fn from(value: String) -> Self {
                match value.as_str() {
                    $($name => Self::$node_type,)*
                    name => Self::Unknown(name.into()),
                }
            }
        }

        impl fmt::Display for RegularNodeType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
                let name = match self {
                    RegularNodeType::Unknown(name) => name,
                    $(RegularNodeType::$node_type => $name,)*
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

conversions!(
    [Anchor, "a"],
    [Animate, "animate"],
    [AnimateMotion, "animateMotion"],
    [AnimateTransform, "animateTransform"],
    [Audio, "audio"],
    [Canvas, "canvas"],
    [Circle, "circle"],
    [ClipPath, "clipPath"],
    [Defs, "defs"],
    [Description, "desc"],
    [Discard, "discard"],
    [Ellipse, "ellipse"],
    [FeBlend, "feBlend"],
    [FeColorMatrix, "feColorMatrix"],
    [FeComponentTransfer, "feComponentTransfer"],
    [FeComposite, "feComposite"],
    [FeConvolveMatrix, "feConvolveMatrix"],
    [FeDiffuseLighting, "feDiffuseLighting"],
    [FeDisplacementMap, "feDisplacementMap"],
    [FeDistantLight, "feDistantLight"],
    [FeDropShadow, "feDropShadow"],
    [FeFlood, "feFlood"],
    [FeFuncA, "feFuncA"],
    [FeFuncB, "feFuncB"],
    [FeFuncG, "feFuncG"],
    [FeFuncR, "feFuncR"],
    [FeGaussianBlur, "feGaussianBlur"],
    [FeImage, "feImage"],
    [FeMerge, "feMerge"],
    [FeMergeNode, "feMergeNode"],
    [FeMorphology, "feMorphology"],
    [FeOffset, "feOffset"],
    [FePointLight, "fePointLight"],
    [FeSpecularLighting, "feSpecularLighting"],
    [FeSpotLight, "feSpotLight"],
    [FeTile, "feTile"],
    [FeTurbulence, "feTurbulence"],
    [Filter, "filter"],
    [ForeignObject, "foreignObject"],
    [Group, "g"],
    [IFrame, "iframe"],
    [Image, "image"],
    [Line, "line"],
    [LinearGradient, "linearGradient"],
    [Marker, "marker"],
    [Mask, "mask"],
    [Metadata, "metadata"],
    [MotionPath, "mpath"],
    [Path, "path"],
    [Pattern, "pattern"],
    [Polygon, "polygon"],
    [Polyline, "polyline"],
    [RadialGradient, "radialGradient"],
    [Rectangle, "rect"],
    [Script, "script"],
    [Set, "set"],
    [Stop, "stop"],
    [Style, "style"],
    [Svg, "svg"],
    [Switch, "switch"],
    [Symbol, "symbol"],
    [Text, "text"],
    [TextPath, "textPath"],
    [Title, "title"],
    [TSpan, "tspan"],
    [Use, "use"],
    [Video, "video"],
    [View, "view"]
);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ChildlessNodeType {
    ProcessingInstruction(String, Option<String>),
    Comment(String),
    Text(String, bool),
}

impl ChildlessNodeType {
    fn tag(self) -> XmlEvent {
        match self {
            Self::ProcessingInstruction(name, data) => {
                XmlEvent::ProcessingInstruction { name, data }
            }
            Self::Comment(text) => XmlEvent::Comment(text),
            Self::Text(text, is_cdata) => {
                if is_cdata {
                    XmlEvent::CData(text)
                } else {
                    XmlEvent::Characters(text)
                }
            }
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
                namespace,
                attributes,
                children,
            } => {
                let (start_tag, end_tag) = node_type.tags(namespace, attributes);
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
                    // starting iteration
                    Some(start_tag)
                } else if let Some(iter) = child_iter {
                    // currently iterating over a child
                    if let Some(element) = iter.next() {
                        Some(element)
                    } else {
                        // switch to next child
                        *child_iter = None;
                        self.next()
                    }
                } else if let Some(child) = children_iter.next() {
                    // not iterating over a child, take next child
                    *child_iter = Some(Box::new(child.into_iter()));
                    self.next()
                } else {
                    // ran out of children, end
                    end_tag.take()
                }
            }
            Self::ChildlessNodeIter { tag } => tag.take(),
        }
    }
}
