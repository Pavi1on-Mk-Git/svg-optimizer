use super::common::iter::EasyIter;
use crate::node::Node;

fn remove_empty_attributes_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes: attributes.filter_to_vec(|attribute| !attribute.value.is_empty()),
            children: remove_empty_attributes(children),
        },
        other => other,
    }
}

pub(crate) fn remove_empty_attributes(nodes: Vec<Node>) -> Vec<Node> {
    nodes.map_to_vec(remove_empty_attributes_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_empty_attributes,
        remove_empty_attributes,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100" height="">
            <rect id="nestedRect" x="10" y="" width="100" height="100"/>
        </rect>
        <rect id="mediumRect" x="" y="10" width="" height="100"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100">
            <rect id="nestedRect" x="10" width="100" height="100"/>
        </rect>
        <rect id="mediumRect" y="10" height="100"/>
        </svg>
        "#
    );
}
