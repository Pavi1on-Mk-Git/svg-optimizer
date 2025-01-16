use super::common::iter::EasyIter;
use crate::node::Node;
use anyhow::Result;

fn sort_attributes_from_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            mut attributes,
            children,
        } => {
            attributes.sort_unstable_by(|fst, snd| fst.name.local_name.cmp(&snd.name.local_name));
            Node::RegularNode {
                node_type,
                attributes,
                children: children.map(sort_attributes_from_node),
            }
        }
        other => other,
    }
}

pub fn sort_attributes(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(nodes.map(sort_attributes_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_sort_attributes,
        sort_attributes,
        r#"
        <svg viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
        <ellipse rx="50" cx="100" ry="50" cy="50"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100">
        <ellipse cx="100" cy="50" rx="50" ry="50"/>
        </svg>
        "#
    );
}
