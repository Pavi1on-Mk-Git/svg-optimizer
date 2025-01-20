use super::common::{constants::*, iter::EasyIter};
use crate::node::{Node, RegularNodeType};
use xml::attribute::OwnedAttribute;

pub const NO_GROUP_ATTRIBUTES: [&str; 11] = [
    ID_NAME,
    CX_NAME,
    CY_NAME,
    HEIGHT_NAME,
    WIDTH_NAME,
    X_NAME,
    Y_NAME,
    R_NAME,
    RX_NAME,
    RY_NAME,
    PATH_DATA_NAME,
];

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
        .filter_to_vec(|attr| !NO_GROUP_ATTRIBUTES.contains(&attr.name.local_name.as_str()))
}

fn remove_common_attributes(nodes: Vec<Node>, common_attributes: &[OwnedAttribute]) -> Vec<Node> {
    nodes.map_to_vec(|node| match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes: attributes.filter_to_vec(|attr| !common_attributes.contains(attr)),
            children,
        },
        other => other,
    })
}

fn extract_common_attributes_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type: RegularNodeType::Group,
            namespace,
            mut attributes,
            children,
        } => {
            let common_attributes = find_common_attributes(&children);
            let children = remove_common_attributes(children, &common_attributes);

            attributes = attributes.filter_to_vec(|attr| {
                common_attributes
                    .iter()
                    .all(|common_attr| common_attr.name != attr.name)
            });
            attributes.extend(common_attributes);

            Node::RegularNode {
                node_type: RegularNodeType::Group,
                namespace,
                attributes,
                children,
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
            children: extract_common_attributes(children),
        },
        other => other,
    }
}

pub fn extract_common_attributes(nodes: Vec<Node>) -> Vec<Node> {
    nodes.map_to_vec(extract_common_attributes_from_node)
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
        <g fill="white" stroke-width="5" stroke="green">
        <circle cx="40" cy="40" r="25"/>
        <circle cx="60" cy="60" r="25"/>
        </g>
        </svg>
        "#
    );

    test_optimize!(
        test_extract_common_attributes_overwrite,
        extract_common_attributes,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
        <g fill="white" stroke="red">
        <circle cx="40" cy="40" r="25" stroke-width="5" stroke="green"/>
        <circle cx="60" cy="60" r="25" stroke-width="5" stroke="green"/>
        </g>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        <g fill="white" stroke-width="5" stroke="green">
        <circle cx="40" cy="40" r="25"/>
        <circle cx="60" cy="60" r="25"/>
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
        <circle cx="60" cy="60" r="25"/>
        </g>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        <g fill="white">
        <circle cx="40" cy="40" r="25"/>
        <circle cx="60" cy="60" r="25"/>
        <circle cx="60" cy="60" r="25"/>
        </g>
        </svg>
        "#
    );
}
