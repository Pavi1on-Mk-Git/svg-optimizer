use super::common::{constants::TRANSFORM_NAME, id_usage::find_attribute_mut, iter::EasyIter};
use crate::node::Node;
use itertools::Itertools;
use lazy_regex::regex;
use nalgebra::Matrix3;

fn create_matrix(arguments: &str) -> Option<Matrix3<f64>> {
    let arguments: Vec<f64> = arguments
        .split_whitespace()
        .filter_map(|arg| arg.parse::<f64>().ok())
        .collect();

    if let [a, b, c, d, e, f] = arguments[..] {
        Some(Matrix3::from_vec(vec![a, b, 0., c, d, 0., e, f, 1.]))
    } else {
        None
    }
}

fn string_to_matrix(matrix_string: &str) -> Option<Matrix3<f64>> {
    println!("{:?}", matrix_string);
    let match_transform = regex!(r"(\w+)\((.+)");

    let captures = match_transform.captures(matrix_string)?;
    let transform_type = captures.get(1)?.as_str();
    let transform_arguments = captures.get(2)?.as_str();

    match transform_type {
        "matrix" => {
            let mat = create_matrix(transform_arguments);
            println!("{:?}", mat);
            mat
        }
        _ => None,
    }
}

fn matrix_to_string(matrix: &Matrix3<f64>) -> String {
    format!(
        "matrix({} {} {} {} {} {})",
        matrix.index((0, 0)),
        matrix.index((1, 0)),
        matrix.index((0, 1)),
        matrix.index((1, 1)),
        matrix.index((0, 2)),
        matrix.index((1, 2))
    )
}

fn merge_transform_attribute(transform_str: &str) -> String {
    let transform_string = transform_str.split_whitespace().join(" ");
    println!("{:?}", transform_string);
    let mut result: Matrix3<f64> = Matrix3::identity();

    for transform in transform_string
        .split(")")
        .filter(|substring| !substring.trim().is_empty())
    {
        if let Some(transform) = string_to_matrix(transform) {
            result *= transform;
        } else {
            return transform_str.into();
        }
    }

    matrix_to_string(&result)
}

fn merge_transforms_in_node(node: Node) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            mut attributes,
            children,
        } => {
            if let Some(transform) = find_attribute_mut(&mut attributes, TRANSFORM_NAME) {
                *transform = merge_transform_attribute(transform);
            }

            Node::RegularNode {
                node_type,
                namespace,
                attributes,
                children: merge_transforms(children),
            }
        }
        other => other,
    }
}

pub fn merge_transforms(nodes: Vec<Node>) -> Vec<Node> {
    nodes.map_to_vec(merge_transforms_in_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_merge_two_matrices,
        merge_transforms,
        r##"<svg viewBox="-40 0 150 100">
        <g transform="matrix(2 3 1 2 1 2) matrix(1 4 2 5 3 6)">
            <path d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"/>
        </g>
        </svg>"##,
        r##"<svg viewBox="-40 0 150 100">
        <g transform="matrix(6 11 9 16 13 23)">
            <path d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"/>
        </g>
        </svg>"##
    );
}
