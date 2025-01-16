use super::common::iter::EasyIter;
use crate::node::{Node, RegularNodeType};
use anyhow::Result;
use xml::attribute::OwnedAttribute;

fn find_common_attributes(nodes: &[Node]) -> Vec<OwnedAttribute> {
    let mut common_attributes = Vec::new();
    let mut found_regular_node = false;
    for node in nodes {
        if let Node::RegularNode { attributes, .. } = node {
            if found_regular_node {
                common_attributes.retain(|attr| attributes.contains(attr));
            } else {
                common_attributes.extend(attributes.clone());
                found_regular_node = true;
            }
        }
    }
    common_attributes
}

fn remove_common_attributes(nodes: Vec<Node>, common_attributes: &[OwnedAttribute]) -> Vec<Node> {
    nodes.map(|node| match node {
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            attributes: attributes.filter(|attr| !common_attributes.contains(attr)),
            children,
        },
        other => other,
    })
}

fn extract_common_attributes_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Group,
            mut attributes,
            children,
        } => {
            let common_attributes = find_common_attributes(&children);
            let children = remove_common_attributes(children, &common_attributes);

            let to_add =
                common_attributes.filter(|attr| !attributes.iter().any(|a| a.name == attr.name));
            attributes.extend(to_add);

            Node::RegularNode {
                node_type: RegularNodeType::Group,
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
            children: children.map(extract_common_attributes_from_node),
        },
        other => other,
    }
}

pub fn extract_common_attributes(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.map(extract_common_attributes_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_extract_common_attributes,
        extract_common_attributes,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
        <g fill="white">
        <circle cx="40" cy="40" r="25" stroke-width="5" stroke="green"/>
        <circle cx="60" cy="60" r="25" stroke-width="5" stroke="green"/>
        </g>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        <g fill="white" r="25" stroke-width="5" stroke="green">
        <circle cx="40" cy="40"/>
        <circle cx="60" cy="60"/>
        </g>
        </svg>
        "#
    );

    test_optimize!(
        test_extract_common_attributes_overwrite,
        extract_common_attributes,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
        <g fill="white" r="20">
        <circle cx="40" cy="40" r="25" stroke-width="5" stroke="green"/>
        <circle cx="60" cy="60" r="25" stroke-width="5" stroke="green"/>
        </g>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        <g fill="white" r="20" stroke-width="5" stroke="green">
        <circle cx="40" cy="40"/>
        <circle cx="60" cy="60"/>
        </g>
        </svg>
        "#
    );

    test_optimize!(
        test_extract_common_attributes_no_extraction,
        extract_common_attributes,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
        <g fill="white">
        <circle cx="40" cy="40" r="25"/>
        <circle cx="60" cy="60" r="25"/>
        <circle cx="60" cy="60" r="20"/>
        </g>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        <g fill="white">
        <circle cx="40" cy="40" r="25"/>
        <circle cx="60" cy="60" r="25"/>
        <circle cx="60" cy="60" r="20"/>
        </g>
        </svg>
        "#
    );
}
