use super::apply_option;
use crate::node::{Node, RegularNodeType};
use anyhow::Result;

fn remove_descriptions_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type:
                RegularNodeType::Description | RegularNodeType::Metadata | RegularNodeType::Title,
            ..
        } => None,
        Node::RegularNode {
            node_type,
            attributes,
            children,
        } => Some(Node::RegularNode {
            node_type,
            attributes,
            children: apply_option(children, remove_descriptions_from_node),
        }),
        other => Some(other),
    }
}

pub fn remove_descriptions(nodes: Vec<Node>) -> Result<Vec<Node>> {
    Ok(apply_option(nodes, remove_descriptions_from_node))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_descriptions,
        remove_descriptions,
        r#"
<desc> You cannot describe me into nonexistence! </desc>
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100" height="100">
            <title> Some Title </title><rect id="nestedRect" x="10" y="10" width="100" height="100"/>
        </rect><metadata> <style/> </metadata>
        <rect id="hugeRect" x="10" y="10" width="100" height="100"/>
        </svg>
        "#,
        r#"

        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100" height="100">
            <rect id="nestedRect" x="10" y="10" width="100" height="100"/>
        </rect>
        <rect id="hugeRect" x="10" y="10" width="100" height="100"/>
        </svg>
        "#
    );
}
