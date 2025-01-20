use super::common::{
    constants::TRANSFORM_NAME, id_usage::find_attribute_mut, iter::EasyIter, unit::round_float,
};
use crate::node::Node;
use itertools::Itertools;
use lazy_regex::regex;
use nalgebra::{Matrix2, Matrix3, Vector2};

fn matrix(arguments: Vec<f64>) -> Option<Matrix3<f64>> {
    if let [a, b, c, d, e, f] = arguments[..] {
        Some(Matrix3::from_vec(vec![a, b, 0., c, d, 0., e, f, 1.]))
    } else {
        None
    }
}

fn translate(arguments: Vec<f64>) -> Option<Matrix3<f64>> {
    match arguments[..] {
        [x, y] => Some(Matrix3::new_translation(&Vector2::new(x, y))),
        [x] => Some(Matrix3::new_translation(&Vector2::new(x, x))),
        _ => None,
    }
}

fn scale(arguments: Vec<f64>) -> Option<Matrix3<f64>> {
    match arguments[..] {
        [x, y] => Some(Matrix3::new_nonuniform_scaling(&Vector2::new(x, y))),
        [x] => Some(Matrix3::new_scaling(x)),
        _ => None,
    }
}

fn rotate(arguments: Vec<f64>) -> Option<Matrix3<f64>> {
    match arguments[..] {
        [a, x, y] => Some(
            Matrix3::new_translation(&Vector2::new(x, y))
                * Matrix3::new_rotation(a.to_radians())
                * Matrix3::new_translation(&Vector2::new(-x, -y)),
        ),
        [a] => Some(Matrix3::new_rotation(a.to_radians())),
        _ => None,
    }
}

fn skew_x(arguments: Vec<f64>) -> Option<Matrix3<f64>> {
    if let [a] = arguments[..] {
        Some(Matrix2::from_vec(vec![1., 0., a.to_radians().tan(), 1.]).to_homogeneous())
    } else {
        None
    }
}

fn skew_y(arguments: Vec<f64>) -> Option<Matrix3<f64>> {
    if let [a] = arguments[..] {
        Some(Matrix2::from_vec(vec![1., a.to_radians().tan(), 0., 1.]).to_homogeneous())
    } else {
        None
    }
}

fn string_to_matrix(matrix_string: &str) -> Option<Matrix3<f64>> {
    let match_transform = regex!(r"(\w+)\((.+)");

    let captures = match_transform.captures(matrix_string)?;
    let transform_type = captures.get(1)?.as_str();
    let transform_arguments = captures.get(2)?.as_str();

    let transform_arguments: Vec<f64> = transform_arguments
        .split_whitespace()
        .filter_map(|arg| arg.parse::<f64>().ok())
        .collect();

    match transform_type {
        "matrix" => matrix(transform_arguments),
        "translate" => translate(transform_arguments),
        "scale" => scale(transform_arguments),
        "rotate" => rotate(transform_arguments),
        "skewX" => skew_x(transform_arguments),
        "skewY" => skew_y(transform_arguments),
        _ => None, // not handling CSS transform functions
    }
}

fn matrix_to_string(matrix: &Matrix3<f64>, precision: usize) -> String {
    format!(
        "matrix({} {} {} {} {} {})",
        round_float(*matrix.index((0, 0)), precision),
        round_float(*matrix.index((1, 0)), precision),
        round_float(*matrix.index((0, 1)), precision),
        round_float(*matrix.index((1, 1)), precision),
        round_float(*matrix.index((0, 2)), precision),
        round_float(*matrix.index((1, 2)), precision)
    )
}

fn merge_transform_attribute(transform_str: &str, precision: usize) -> String {
    let transform_string = transform_str.split_whitespace().join(" ");
    let mut result: Matrix3<f64> = Matrix3::identity();

    for transform in transform_string
        .split(")")
        .filter(|substring| !substring.trim().is_empty())
    {
        if let Some(transform) = string_to_matrix(transform) {
            println!("{:?}", transform);
            result *= transform;
        } else {
            return transform_str.into();
        }
    }

    matrix_to_string(&result, precision)
}

fn merge_transforms_in_node(node: Node, precision: usize) -> Node {
    match node {
        Node::RegularNode {
            node_type,
            namespace,
            mut attributes,
            children,
        } => {
            if let Some(transform) = find_attribute_mut(&mut attributes, TRANSFORM_NAME) {
                *transform = merge_transform_attribute(transform, precision);
            }

            Node::RegularNode {
                node_type,
                namespace,
                attributes,
                children: merge_transforms(children, precision),
            }
        }
        other => other,
    }
}

pub fn merge_transforms(nodes: Vec<Node>, precision: usize) -> Vec<Node> {
    nodes.map_to_vec(|node| merge_transforms_in_node(node, precision))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    fn test_merge(nodes: Vec<Node>) -> Vec<Node> {
        merge_transforms(nodes, 2)
    }

    test_optimize!(
        test_merge_two_matrices,
        test_merge,
        r##"<svg viewBox="-40 0 150 100">
        <g transform="translate(10 10) matrix(2 3 1 2 1 2) matrix(1 4 2 5 3 6)">
            <path d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"/>
        </g>
        </svg>"##,
        r##"<svg viewBox="-40 0 150 100">
        <g transform="matrix(6 11 9 16 23 33)">
            <path d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"/>
        </g>
        </svg>"##
    );
}
