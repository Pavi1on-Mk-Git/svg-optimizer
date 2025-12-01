use super::common::{
    constants::{HEIGHT_NAME, VIEWBOX_NAME, WIDTH_NAME},
    id_usage::find_attribute,
    iter::EasyIter,
    unit::find_and_convert_to_px,
};
use crate::node::{Node, RegularNodeType};
use itertools::Itertools;
use xml::attribute::OwnedAttribute;

fn get_dimensions(attributes: &[OwnedAttribute]) -> (Option<f64>, Option<f64>) {
    (
        find_and_convert_to_px(attributes, WIDTH_NAME),
        find_and_convert_to_px(attributes, HEIGHT_NAME),
    )
}

fn remove_dimensions_in_attributes(attributes: Vec<OwnedAttribute>) -> Vec<OwnedAttribute> {
    if let (Some(viewbox), (Some(width), Some(height))) = (
        find_attribute(&attributes, VIEWBOX_NAME),
        get_dimensions(&attributes),
    ) {
        let expected_viewbox = format!("0 0 {width} {height}");

        if expected_viewbox == viewbox.split_whitespace().join(" ") {
            return attributes.filter_to_vec(|attr| {
                let name = &attr.name.local_name;
                name != WIDTH_NAME && name != HEIGHT_NAME
            });
        }
    }
    attributes
}

fn remove_dimensions_from_node(node: Node) -> Node {
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
            attributes: remove_dimensions_in_attributes(attributes),
            children: remove_dimensions(children),
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
            children: remove_dimensions(children),
        },
        other @ Node::ChildlessNode { .. } => other,
    }
}

pub(crate) fn remove_dimensions(nodes: Vec<Node>) -> Vec<Node> {
    nodes.map_to_vec(remove_dimensions_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_dimensions,
        remove_dimensions,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="200" height="200" viewBox="  0  0 200 200  ">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="  0  0 200 200  ">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#
    );
    test_optimize!(
        test_remove_dimensions_no_remove,
        remove_dimensions,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 200">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 200 200">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#
    );
}
