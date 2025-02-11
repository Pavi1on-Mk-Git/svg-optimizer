use xml::attribute::OwnedAttribute;

use super::common::iter::EasyIter;
use crate::node::{Node, NodeNamespace};

const ALLOWED_NAMESPACES: [&str; 3] = ["", "xlink", "xml"];

fn remove_editor_namespace_data_from_namespace(mut namespace: NodeNamespace) -> NodeNamespace {
    namespace
        .element_namespace
        .0
        .retain(|key, _| ALLOWED_NAMESPACES.contains(&key.as_str()));
    NodeNamespace {
        parent_namespace: namespace.parent_namespace,
        prefix: namespace.prefix,
        element_namespace: namespace.element_namespace,
    }
}

fn has_editor_namespace_data(attr: &OwnedAttribute) -> bool {
    attr.name
        .prefix
        .as_ref()
        .map_or(true, |prefix| ALLOWED_NAMESPACES.contains(&prefix.as_str()))
}

fn remove_editor_namespace_data_from_node(node: Node) -> Option<Node> {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => namespace.prefix.is_none().then_some(Node::RegularNode {
            node_type,
            namespace: remove_editor_namespace_data_from_namespace(namespace),
            attributes: attributes.filter_to_vec(has_editor_namespace_data),
            children: children.filter_map_to_vec(remove_editor_namespace_data_from_node),
        }),
        other => Some(other),
    }
}

pub fn remove_editor_namespace_data(nodes: Vec<Node>) -> Vec<Node> {
    nodes.filter_map_to_vec(remove_editor_namespace_data_from_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_editor_namespace_data,
        remove_editor_namespace_data,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape">
        <marker inkscape:stockid="Arrow2Lend" orient="auto" refY="0.0" refX="0.0" id="Arrow2Lend" style="overflow:visible;"/><sodipodi:namedview
        pagecolor="#ffffff"
        bordercolor="#666666"
        borderopacity="1"
        objecttolerance="10"
        gridtolerance="10"
        guidetolerance="10"
        inkscape:pageopacity="0"
        inkscape:pageshadow="2"
        inkscape:window-width="991"
        inkscape:window-height="606"
        id="namedview62"
        showgrid="false"
        inkscape:zoom="3.0752688"
        inkscape:cx="139.5"
        inkscape:cy="40.5"
        inkscape:window-x="891"
        inkscape:window-y="177"
        inkscape:window-maximized="0"
        inkscape:current-layer="g4" />
        </svg>
        "##,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <marker orient="auto" refY="0.0" refX="0.0" id="Arrow2Lend" style="overflow:visible;"/>
        </svg>
        "#
    );
}
