use super::common::{
    constants::{PATH_DATA_NAME, PATH_LENGTH_NAME},
    id_usage::{find_attribute, find_attribute_mut},
    unit::{convert_to_px, find_and_convert_to_px},
};
use crate::node::{Node, RegularNodeType};
use itertools::Either;
use xml::attribute::OwnedAttribute;
use Either::{Left, Right};

fn path_has_same_attr(attribute: &OwnedAttribute, attributes: &[OwnedAttribute]) -> bool {
    attributes.iter().any(|second_attribute| {
        attribute == second_attribute
            || (attribute.name == second_attribute.name
                && (attribute.name.local_name == PATH_DATA_NAME
                    || attribute.name.local_name == PATH_LENGTH_NAME))
    })
}

fn path_attributes_equal(first: &[OwnedAttribute], second: &[OwnedAttribute]) -> bool {
    first.len() == second.len()
        && first
            .iter()
            .all(|first_attribute| path_has_same_attr(first_attribute, second))
}

fn merge_children_paths(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children,
        } => Node::RegularNode {
            node_type,
            namespace,
            attributes,
            children: merge_consecutive_paths(children),
        },
        other => other,
    }
}

fn merge_path_data(fst_attrs: &[OwnedAttribute], snd_attrs: &mut [OwnedAttribute]) {
    let fst_path_data = find_attribute(fst_attrs, PATH_DATA_NAME);
    let snd_path_data = find_attribute_mut(snd_attrs, PATH_DATA_NAME);

    if let (Some(fst_path_data), Some(snd_path_data)) = (fst_path_data, snd_path_data) {
        *snd_path_data = format!(
            "{} {}",
            fst_path_data.trim_end(),
            snd_path_data.trim_start()
        );
    }
}

fn merge_path_len(fst_attrs: &[OwnedAttribute], snd_attrs: &mut [OwnedAttribute]) {
    let fst_path_len = find_and_convert_to_px(fst_attrs, PATH_LENGTH_NAME);
    let snd_path_len_attr = find_attribute_mut(snd_attrs, PATH_LENGTH_NAME);

    if let (Some(fst_path_len), Some(snd_path_len_attr)) = (fst_path_len, snd_path_len_attr) {
        if let Some(snd_path_len) = convert_to_px(snd_path_len_attr) {
            *snd_path_len_attr = format!("{}", fst_path_len + snd_path_len);
        }
    }
}

fn merge_paths(first: Node, second: Node) -> Either<(Node, Node), Node> {
    match (first, second) {
        (
            Node::RegularNode {
                node_type: RegularNodeType::Path,
                namespace: fst_namespace,
                attributes: fst_attrs,
                children: mut fst_children,
            },
            Node::RegularNode {
                node_type: RegularNodeType::Path,
                namespace: snd_namespace,
                attributes: mut snd_attrs,
                children: mut snd_children,
            },
        ) => {
            fst_children = merge_consecutive_paths(fst_children);
            snd_children = merge_consecutive_paths(snd_children);

            if fst_namespace == snd_namespace
                && fst_children == snd_children
                && path_attributes_equal(&fst_attrs, &snd_attrs)
            {
                merge_path_data(&fst_attrs, &mut snd_attrs);
                merge_path_len(&fst_attrs, &mut snd_attrs);

                Right(Node::RegularNode {
                    node_type: RegularNodeType::Path,
                    namespace: snd_namespace,
                    attributes: snd_attrs,
                    children: snd_children,
                })
            } else {
                Left((
                    Node::RegularNode {
                        node_type: RegularNodeType::Path,
                        namespace: fst_namespace,
                        attributes: fst_attrs,
                        children: fst_children,
                    },
                    Node::RegularNode {
                        node_type: RegularNodeType::Path,
                        namespace: snd_namespace,
                        attributes: snd_attrs,
                        children: snd_children,
                    },
                ))
            }
        }
        (first, second) => Left((merge_children_paths(first), merge_children_paths(second))),
    }
}

pub fn merge_consecutive_paths(nodes: Vec<Node>) -> Vec<Node> {
    let mut result = vec![];
    let mut node_holder = None;
    for node in nodes {
        node_holder = Some(if let Some(prev_node) = node_holder {
            match merge_paths(prev_node, node) {
                Left((first, second)) => {
                    result.push(first);
                    second
                }
                Right(merged) => merged,
            }
        } else {
            merge_children_paths(node)
        })
    }
    if let Some(prev_node) = node_holder {
        result.push(prev_node);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_merge_consecutive_paths,
        merge_consecutive_paths,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <path pathLength="3px" fill="red" stroke="black" stroke-width="5" transform="translate(-1100 -1200)" stroke-dashoffset="148" stroke-dasharray="148 148" d="M1400 1520 L1260 1480"/><path pathLength="4pt" fill="red" stroke="black" stroke-width="5" transform="translate(-1100 -1200)" stroke-dashoffset="148" stroke-dasharray="148 148" d="M1280 480 L1110 460 L1060 260 L1180 240"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <path pathLength="8" fill="red" stroke="black" stroke-width="5" transform="translate(-1100 -1200)" stroke-dashoffset="148" stroke-dasharray="148 148" d="M1400 1520 L1260 1480 M1280 480 L1110 460 L1060 260 L1180 240"/>
        </svg>
        "#
    );
}
