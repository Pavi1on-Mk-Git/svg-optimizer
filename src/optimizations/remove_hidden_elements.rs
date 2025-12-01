#[allow(clippy::wildcard_imports)]
use super::common::{constants::*, iter::EasyIter, unit::find_and_convert_to_px};
use crate::node::{Node, RegularNodeType};
use xml::attribute::OwnedAttribute;

fn is_no_display(attr: &OwnedAttribute) -> bool {
    attr.name.local_name == DISPLAY_NAME && attr.value == NONE_VAL
}

fn is_opacity_zero(attr: &OwnedAttribute) -> bool {
    attr.name.local_name == OPACITY_NAME && attr.value == "0"
}

fn remove_no_display(node: &Node) -> bool {
    match node {
        Node::RegularNode { attributes, .. } => attributes
            .iter()
            .any(|attr| is_no_display(attr) || is_opacity_zero(attr)),
        Node::ChildlessNode { .. } => false,
    }
}

fn remove_circle(node: &Node) -> bool {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Circle,
            attributes,
            ..
        } => find_and_convert_to_px(attributes, R_NAME).unwrap_or(0.) == 0.,
        Node::RegularNode { .. } | Node::ChildlessNode { .. } => false,
    }
}

fn remove_ellipse(node: &Node) -> bool {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Ellipse,
            attributes,
            ..
        } => {
            let rx = find_and_convert_to_px(attributes, RX_NAME);
            let ry = find_and_convert_to_px(attributes, RY_NAME);

            rx == Some(0.) || ry == Some(0.)
        }
        Node::RegularNode { .. } | Node::ChildlessNode { .. } => false,
    }
}

fn remove_zero_dimensions(node: &Node) -> bool {
    match node {
        Node::RegularNode {
            node_type:
                RegularNodeType::Rectangle | RegularNodeType::Pattern | RegularNodeType::Image,
            attributes,
            ..
        } => {
            let width = find_and_convert_to_px(attributes, WIDTH_NAME).unwrap_or(0.);
            let height = find_and_convert_to_px(attributes, HEIGHT_NAME).unwrap_or(0.);

            width == 0. || height == 0.
        }
        Node::RegularNode { .. } | Node::ChildlessNode { .. } => false,
    }
}

fn is_empty_attr(attributes: &[OwnedAttribute], name: &str) -> bool {
    attributes
        .iter()
        .all(|attr| attr.name.local_name != name || attr.value.trim() == "")
}

fn remove_empty_data(node: &Node) -> bool {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Path,
            attributes,
            ..
        } => is_empty_attr(attributes, PATH_DATA_NAME),
        Node::RegularNode {
            node_type: RegularNodeType::Polygon | RegularNodeType::Polyline,
            attributes,
            ..
        } => is_empty_attr(attributes, POINTS_NAME),
        Node::RegularNode { .. } | Node::ChildlessNode { .. } => false,
    }
}

fn should_remove(node: &Node) -> bool {
    remove_no_display(node)
        || remove_circle(node)
        || remove_ellipse(node)
        || remove_zero_dimensions(node)
        || remove_empty_data(node)
}

fn remove_hidden_elements_from_node(node: Node) -> Option<Node> {
    (!should_remove(&node)).then_some(match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: remove_hidden_elements(children),
        },
        other @ Node::ChildlessNode { .. } => other,
    })
}

pub(crate) fn remove_hidden_elements(nodes: Vec<Node>) -> Vec<Node> {
    nodes.filter_map_to_vec(remove_hidden_elements_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_hidden_display_none,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><ellipse rx="50" cx="100" ry="50" cy="50" display="none"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );

    test_optimize!(
        test_remove_hidden_opacity_0,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><ellipse rx="50" cx="100" ry="50" cy="50" opacity="0"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );

    test_optimize!(
        test_remove_hidden_circle,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><circle r="0" cx="100" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );

    test_optimize!(
        test_remove_hidden_circle_no_radius,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><circle cx="100" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );

    test_optimize!(
        test_remove_hidden_ellipse,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><ellipse rx="0" cx="100" ry="50" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );

    test_optimize!(
        test_remove_hidden_zero_dimensions,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><rect width="300" height="0" x="10" y="10"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );

    test_optimize!(
        test_remove_hidden_zero_dimensions_no_dimension,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><rect height="100" x="10" y="10"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );

    test_optimize!(
        test_remove_hidden_empty_path,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><path d="        "/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );

    test_optimize!(
        test_remove_hidden_empty_poly,
        remove_hidden_elements,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg"><polygon points="        "/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        </svg>
        "#
    );
}
