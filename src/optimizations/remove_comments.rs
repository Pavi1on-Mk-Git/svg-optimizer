use super::apply_to_nodes;
use crate::node::ChildlessNodeType;
use crate::node::Node;

fn remove_comments_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            attributes,
            children: remove_comments(children),
        }),
        Node::ChildlessNode {
            node_type: ChildlessNodeType::Comment(_),
        } => None,
        childless_node => Some(childless_node),
    }
}

pub fn remove_comments(nodes: Vec<Node>) -> Vec<Node> {
    apply_to_nodes(nodes, remove_comments_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_comments,
        remove_comments,
        r#"
        <!-- comment -->
        <svg xmlns="http://www.w3.org/2000/svg"><!-- comment --></svg>
        <!-- comment -->
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg"/>
        "#
    );
}
