use super::common::iter::EasyIter;
use crate::node::{Node, RegularNodeType};
use anyhow::Result;
use xml::{attribute::OwnedAttribute, name::OwnedName};

const WIDTH_NAME: &str = "width";
const HEIGHT_NAME: &str = "height";
const VIEWBOX_NAME: &str = "viewBox";

fn get_dimensions(attributes: &[OwnedAttribute]) -> (Option<f64>, Option<f64>) {
    let find_float_attr = |name: &str| {
        attributes
            .iter()
            .find(|attribute| attribute.name.local_name == name)
            .and_then(|attr| attr.value.parse::<f64>().ok())
    };

    (find_float_attr(WIDTH_NAME), find_float_attr(HEIGHT_NAME))
}

fn convert_to_viewbox_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type:
                node_type @ (RegularNodeType::Marker
                | RegularNodeType::Pattern
                | RegularNodeType::Svg(_)
                | RegularNodeType::Symbol
                | RegularNodeType::View),
            mut attributes,
            children,
        } => {
            let (width, height) = get_dimensions(&attributes);

            if let (Some(width), Some(height)) = (width, height) {
                let viewbox_attributes = OwnedAttribute {
                    name: OwnedName {
                        local_name: VIEWBOX_NAME.into(),
                        namespace: None,
                        prefix: None,
                    },
                    value: format!("0 0 {width} {height}"),
                };

                attributes = attributes.filter(|attr| {
                    let name = &attr.name.local_name;
                    name != WIDTH_NAME && name != HEIGHT_NAME
                });
                attributes.push(viewbox_attributes);
            }

            Node::RegularNode {
                node_type,
                attributes,
                children: children.map(convert_to_viewbox_from_node),
            }
        }
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            attributes,
            children: children.map(convert_to_viewbox_from_node),
        },
        other => other,
    }
}

pub fn convert_to_viewbox(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.map(convert_to_viewbox_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_convert_to_viewbox,
        convert_to_viewbox,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#
    );
}
