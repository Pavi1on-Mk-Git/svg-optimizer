use super::common::iter::EasyIter;
use crate::node::{ChildlessNodeType, Node, RegularNodeType};

fn contains_only_whitespace(node: &Node) -> bool {
    if let Node::ChildlessNode {
        node_type: ChildlessNodeType::Text(text, ..),
    } = node
    {
        text.trim().is_empty()
    } else {
        false
    }
}

fn remove_empty_texts_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type: node_type @ (RegularNodeType::Text | RegularNodeType::TSpan),
            namespace,
            attributes,
            children,
        } => {
            let new_children: Vec<Node> = remove_empty_texts(children);
            let should_retain = !new_children.iter().all(contains_only_whitespace);

            should_retain.then_some(Node::RegularNode {
                node_type,
                namespace,
                attributes,
                children: new_children,
            })
        }
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: remove_empty_texts(children),
        }),
        other => Some(other),
    }
}

pub(crate) fn remove_empty_texts(nodes: Vec<Node>) -> Vec<Node> {
    nodes.filter_map_to_vec(remove_empty_texts_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_empty_texts,
        remove_empty_texts,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100" height="100">
            <text>  <tspan/> </text><rect id="nestedRect" x="10" y="10" width="100" height="100"/>
        </rect>
        <text> Sth </text>
        <rect id="hugeRect" x="10" y="10" width="100" height="100"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100" height="100">
            <rect id="nestedRect" x="10" y="10" width="100" height="100"/>
        </rect>
        <text> Sth </text>
        <rect id="hugeRect" x="10" y="10" width="100" height="100"/>
        </svg>
        "#
    );
}
