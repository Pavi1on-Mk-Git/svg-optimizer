use super::common::{
    constants::{RX_NAME, RY_NAME, R_NAME},
    iter::EasyIter,
};
use crate::node::{Node, NodeNamespace, RegularNodeType};
use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;

fn ellipses_to_circles_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Ellipse,
            namespace,
            attributes,
            children,
            ..
        } => get_new_node(namespace, attributes, children),
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: ellipses_to_circles(children),
        },
        childless_node => childless_node,
    }
}

fn get_radii(attributes: &[OwnedAttribute]) -> (Option<&OwnedAttribute>, Option<&OwnedAttribute>) {
    let find_attr = |name: &str| {
        attributes
            .iter()
            .find(|attribute| attribute.name.local_name == name)
    };

    (find_attr(RX_NAME), find_attr(RY_NAME))
}

fn get_new_node(
    namespace: NodeNamespace,
    mut attributes: Vec<OwnedAttribute>,
    children: Vec<Node>,
) -> Node {
    let (rx, ry) = get_radii(&attributes);

    let node_type = match (rx, ry) {
        (Some(rx), Some(ry)) if rx.value == ry.value => {
            let radius_attribute = OwnedAttribute {
                name: OwnedName {
                    local_name: R_NAME.into(),
                    namespace: rx.name.namespace.clone(),
                    prefix: rx.name.prefix.clone(),
                },
                value: rx.value.clone(),
            };

            attributes = attributes.filter_to_vec(|attr| {
                let name = &attr.name.local_name;
                name != RX_NAME && name != RY_NAME
            });
            attributes.push(radius_attribute);

            RegularNodeType::Circle
        }
        _ => RegularNodeType::Ellipse,
    };

    Node::RegularNode {
        node_type,
        namespace,
        attributes,
        children,
    }
}

pub(crate) fn ellipses_to_circles(nodes: Vec<Node>) -> Vec<Node> {
    nodes.map_to_vec(ellipses_to_circles_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_ellipses_to_circles,
        ellipses_to_circles,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
        <ellipse cx="100" cy="50" rx="50" ry="50"/>
        <ellipse cx="100" cy="50" rx="50" ry="50"> Text in here </ellipse>
        <ellipse cx="100" cy="50" rx="60" ry="50"/>
        <ellipse cx="100" cy="50" rx="60"/>
        <ellipse/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        <circle cx="100" cy="50" r="50"/>
        <circle cx="100" cy="50" r="50"> Text in here </circle>
        <ellipse cx="100" cy="50" rx="60" ry="50"/>
        <ellipse cx="100" cy="50" rx="60"/>
        <ellipse/>
        </svg>
        "#
    );
}
