use super::apply_option;
use crate::node::Node;
use crate::node::RegularNodeType;
use anyhow::Result;
use xml::attribute::OwnedAttribute;

fn ellipses_to_circles_from_node(node: Node) -> Option<Node> {
    Some(match node {
        Node::RegularNode {
            node_type: RegularNodeType::Ellipse,
            attributes,
            children,
        } => {
            let children = ellipses_to_circles(children).unwrap();

            let (node_type, attributes) = match circle_attributes(attributes) {
                Ok(attributes) => (RegularNodeType::Circle, attributes),
                Err(attributes) => (RegularNodeType::Ellipse, attributes),
            };

            Node::RegularNode {
                node_type,
                attributes,
                children,
            }
        }
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            attributes,
            children: ellipses_to_circles(children).unwrap(),
        },
        childless_node => childless_node,
    })
}

fn circle_attributes(
    attributes: Vec<OwnedAttribute>,
) -> Result<Vec<OwnedAttribute>, Vec<OwnedAttribute>> {
    let rx_name = "rx";
    let ry_name = "ry";

    let rx = attributes
        .iter()
        .find(|attr| attr.name.local_name == rx_name);
    let ry = attributes
        .iter()
        .find(|attr| attr.name.local_name == ry_name);

    match (rx, ry) {
        (Some(rx), Some(ry)) if rx.value == ry.value => {
            let mut r_name = rx.name.clone();
            r_name.local_name = "r".into();

            let r_val = rx.value.clone();

            let mut attributes: Vec<OwnedAttribute> = attributes
                .into_iter()
                .filter(|attr| {
                    let name = &attr.name.local_name;
                    name != rx_name && name != ry_name
                })
                .collect();
            attributes.push(OwnedAttribute {
                name: r_name,
                value: r_val,
            });
            Ok(attributes)
        }
        _ => Err(attributes),
    }
}

pub fn ellipses_to_circles(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(apply_option(nodes, ellipses_to_circles_from_node))
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
        </svg>
        "#,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
        <circle cx="100" cy="50" r="50"/>
        </svg>
        "#
    );
}
