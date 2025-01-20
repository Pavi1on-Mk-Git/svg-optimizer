use super::common::constants::{
    FILL_NAME, FILL_OPACITY_NAME, NONE_VAL, STROKE_OPACITY_NAME, STROKE_WIDTH_NAME,
};
use super::common::iter::EasyIter;
use super::common::unit::convert_to_px;
use crate::node::Node;
use xml::attribute::OwnedAttribute;

fn is_stroke_opacity_zero(attr: &OwnedAttribute) -> bool {
    attr.name.local_name == STROKE_OPACITY_NAME && attr.value == "0"
}

fn is_stroke_width_zero(attr: &OwnedAttribute) -> bool {
    attr.name.local_name == STROKE_WIDTH_NAME && convert_to_px(&attr.value) == Some(0.)
}

fn is_fill_none(attr: &OwnedAttribute) -> bool {
    attr.name.local_name == FILL_NAME && attr.value == NONE_VAL
}

fn is_fill_opacity_zero(attr: &OwnedAttribute) -> bool {
    attr.name.local_name == FILL_OPACITY_NAME && attr.value == "0"
}

fn is_stroke_invisible(attributes: &[OwnedAttribute]) -> bool {
    attributes
        .iter()
        .any(|attr| is_stroke_opacity_zero(attr) || is_stroke_width_zero(attr))
}

fn is_fill_invisible(attributes: &[OwnedAttribute]) -> bool {
    attributes
        .iter()
        .any(|attr| is_fill_none(attr) || is_fill_opacity_zero(attr))
}

fn remove_useless_stroke_and_fill_from_attributes(
    mut attributes: Vec<OwnedAttribute>,
) -> Vec<OwnedAttribute> {
    if is_stroke_invisible(&attributes) {
        attributes = attributes.filter_to_vec(|attr| !attr.name.local_name.starts_with("stroke"))
    }
    if is_fill_invisible(&attributes) {
        attributes = attributes.filter_to_vec(|attr| !attr.name.local_name.starts_with("fill-"))
    }
    attributes
}

fn remove_useless_stroke_and_fill_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes: remove_useless_stroke_and_fill_from_attributes(attributes),
            children: remove_useless_stroke_and_fill(children),
        },
        other => other,
    }
}

pub fn remove_useless_stroke_and_fill(nodes: Vec<Node>) -> Vec<Node> {
    nodes.map_to_vec(remove_useless_stroke_and_fill_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_useless_stroke_and_fill,
        remove_useless_stroke_and_fill,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <circle cx="150" cy="50" r="40" fill="none" fill-width="10" fill-opacity="1"/>
        <circle cx="150" cy="50" r="40" fill="red" fill-width="10" fill-opacity="0"/>
        <circle cx="150" cy="50" r="40" stroke-width="0" stroke-opacity="1" stroke-linecap="round"/>
        <circle cx="150" cy="50" r="40" stroke-width="0pt" stroke-opacity="1" stroke-linecap="round"/>
        <circle cx="150" cy="50" r="40" stroke-width="3" stroke-opacity="0" stroke-linecap="round"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
        <circle cx="150" cy="50" r="40" fill="none"/>
        <circle cx="150" cy="50" r="40" fill="red"/>
        <circle cx="150" cy="50" r="40"/>
        <circle cx="150" cy="50" r="40"/>
        <circle cx="150" cy="50" r="40"/>
        </svg>
        "#
    );
}
