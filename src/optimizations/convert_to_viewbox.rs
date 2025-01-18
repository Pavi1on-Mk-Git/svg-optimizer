use super::common::constants::{HEIGHT_NAME, WIDTH_NAME};
use super::common::iter::EasyIter;
use super::common::unit::find_and_convert_to_px;
use crate::node::{Node, RegularNodeType};
use anyhow::Result;
use xml::{attribute::OwnedAttribute, name::OwnedName};
const VIEWBOX_NAME: &str = "viewBox";

fn get_dimensions(attributes: &[OwnedAttribute]) -> (Option<f64>, Option<f64>) {
    (
        find_and_convert_to_px(attributes, WIDTH_NAME),
        find_and_convert_to_px(attributes, HEIGHT_NAME),
    )
}

fn convert_to_viewbox_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type:
                node_type @ (RegularNodeType::Marker
                | RegularNodeType::Pattern
                | RegularNodeType::Svg
                | RegularNodeType::Symbol
                | RegularNodeType::View),
            namespace,
            mut attributes,
            children,
            ..
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
                namespace,
                attributes,
                children: children.map(convert_to_viewbox_from_node),
            }
        }
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
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

    test_optimize!(
        test_convert_to_viewbox_other_units,
        convert_to_viewbox,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="200px" height="10pc">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 150">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#
    );
}
