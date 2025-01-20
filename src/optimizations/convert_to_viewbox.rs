use super::common::constants::{HEIGHT_NAME, VIEWBOX_NAME, WIDTH_NAME};
use super::common::id::find_attribute;
use super::common::iter::EasyIter;
use super::common::unit::find_and_convert_to_px;
use crate::node::{Node, RegularNodeType};
use anyhow::Result;
use xml::{attribute::OwnedAttribute, name::OwnedName};

fn get_dimensions(attributes: &[OwnedAttribute]) -> (Option<f64>, Option<f64>) {
    (
        find_and_convert_to_px(attributes, WIDTH_NAME),
        find_and_convert_to_px(attributes, HEIGHT_NAME),
    )
}

fn remove_dimensions(attributes: Vec<OwnedAttribute>) -> Vec<OwnedAttribute> {
    attributes.filter_to_vec(|attr| {
        let name = &attr.name.local_name;
        name != WIDTH_NAME && name != HEIGHT_NAME
    })
}

fn convert_to_viewbox_in_attributes(attributes: Vec<OwnedAttribute>) -> Vec<OwnedAttribute> {
    if find_attribute(&attributes, VIEWBOX_NAME).is_some() {
        return remove_dimensions(attributes);
    }

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

        let mut new_attributes = remove_dimensions(attributes);
        new_attributes.push(viewbox_attributes);
        new_attributes
    } else {
        attributes
    }
}

fn convert_to_viewbox_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Svg,
            namespace,
            attributes,
            children,
            ..
        } => Node::RegularNode {
            node_type: RegularNodeType::Svg,
            namespace,
            attributes: convert_to_viewbox_in_attributes(attributes),
            children: children.map_to_vec(convert_to_viewbox_from_node),
        },
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: children.map_to_vec(convert_to_viewbox_from_node),
        },
        other => other,
    }
}

pub fn convert_to_viewbox(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.map_to_vec(convert_to_viewbox_from_node))
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

    test_optimize!(
        test_convert_to_viewbox_already_exists,
        convert_to_viewbox,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 200">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#
    );
}
