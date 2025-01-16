use super::EasyIter;
use crate::node::{Node, RegularNodeType};
use anyhow::Result;
use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;

fn ellipses_to_circles_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Ellipse,
            attributes,
            children,
        } => get_new_node(attributes, children),
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            attributes,
            children: children.map(ellipses_to_circles_from_node),
        },
        childless_node => childless_node,
    }
}

const RX_NAME: &str = "rx";
const RY_NAME: &str = "ry";

fn get_radii(attributes: &[OwnedAttribute]) -> (Option<&OwnedAttribute>, Option<&OwnedAttribute>) {
    let rx = attributes
        .iter()
        .find(|attribute| attribute.name.local_name == RX_NAME);
    let ry = attributes
        .iter()
        .find(|attribute| attribute.name.local_name == RY_NAME);

    (rx, ry)
}

fn get_new_node(mut attributes: Vec<OwnedAttribute>, children: Vec<Node>) -> Node {
    let (rx, ry) = get_radii(&attributes);

    let node_type = match (rx, ry) {
        (Some(rx), Some(ry)) if rx.value == ry.value => {
            let radius_attribute = OwnedAttribute {
                name: OwnedName {
                    local_name: "r".into(),
                    namespace: rx.name.namespace.clone(),
                    prefix: rx.name.prefix.clone(),
                },
                value: rx.value.clone(),
            };

            attributes = attributes.filter(|attr| {
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
        attributes,
        children,
    }
}

pub fn ellipses_to_circles(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.map(ellipses_to_circles_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::test::test_optimize;
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
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
        <circle cx="100" cy="50" r="50"/>
        <circle cx="100" cy="50" r="50"> Text in here </circle>
        <ellipse cx="100" cy="50" rx="60" ry="50"/>
        <ellipse cx="100" cy="50" rx="60"/>
        <ellipse/>
        </svg>
        "#
    );
}
