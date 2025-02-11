use super::common::{
    id_generator::IdGenerator, id_usage::find_ids_for_subtree, iter::EasyIter,
    replace_ids::replace_ids,
};
use crate::node::Node;
use itertools::Itertools;
use std::collections::BTreeMap;

fn is_hex_color_prefix(id: &str) -> bool {
    id.chars()
        .all(|char| "abcdefABCDEF0123456789".chars().contains(&char))
        && id.len() <= 6
}

fn make_shorten_ids_map(nodes: &Vec<Node>) -> BTreeMap<String, String> {
    let ids = find_ids_for_subtree(nodes).filter_to_vec(|id| !is_hex_color_prefix(id));

    ids.clone()
        .into_iter()
        .zip(IdGenerator::new(vec![]).filter(|id| !ids.contains(id)))
        .collect()
}

pub fn shorten_ids(nodes: Vec<Node>) -> Vec<Node> {
    let id_map = make_shorten_ids_map(&nodes);
    replace_ids(nodes, &id_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::common::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_shorten_ids_with_new_id_existing,
        shorten_ids,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="smallRect1" x="10" y="10" width="100" height="100">
            <rect id="nestedRect" x="10" y="10" width="100" height="100"/>
        </rect>
        <rect id="mediumRect" x="10" y="10" width="100" height="100"/>
        <rect id="largeRect" x="10" y="10" width="100" height="100"/>
        <rect id="g" x="10" y="10" width="100" height="100"/>
        </svg>
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg">
        <rect id="h" x="10" y="10" width="100" height="100">
            <rect id="i" x="10" y="10" width="100" height="100"/>
        </rect>
        <rect id="j" x="10" y="10" width="100" height="100"/>
        <rect id="k" x="10" y="10" width="100" height="100"/>
        <rect id="l" x="10" y="10" width="100" height="100"/>
        </svg>
        "#
    );

    test_optimize!(
        test_shorten_id_references,
        shorten_ids,
        r##"
        <svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
        <style>
            #smallRect {
                stroke: #000066;
                fill: #00cc00;
            }
            #unused {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#smallRect" x="10" fill="blue" />
        <use href="#unused" x="10" fill="blue" />
        <rect id="smallRect" x="10" y="10" width="100" height="100" />
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            #g {
                stroke: #000066;
                fill: #00cc00;
            }
            #unused {
                stroke: #000066;
                fill: #00cc00;
            }
        </style>

        <use href="#g" x="10" fill="blue"/>
        <use href="#unused" x="10" fill="blue"/>
        <rect id="g" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );

    test_optimize!(
        test_shorten_id_references_cdata,
        shorten_ids,
        r##"
        <svg width="120" height="120" viewBox="0 0 120 120" xmlns="http://www.w3.org/2000/svg">
        <style>
            <![CDATA[
            #smallRect {
                stroke: #000066;
                fill: #00cc00;
            }
            ]]>
        </style>

        <use href="#smallRect" x="10" fill="blue" />
        <rect id="smallRect" x="10" y="10" width="100" height="100" />
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            <![CDATA[
            #g {
                stroke: #000066;
                fill: #00cc00;
            }
            ]]>
        </style>

        <use href="#g" x="10" fill="blue"/>
        <rect id="g" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );

    test_optimize!(
        test_shorten_id_same_as_hex_color_not_shortened,
        shorten_ids,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            <![CDATA[
            #aacc00 {
                stroke: #000066;
                fill: #aacc00;
            }
            ]]>
        </style>

        <use href="#aacc00" x="10" fill="blue"/>
        <rect id="aacc00" x="10" y="10" width="100" height="100"/>
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <style>
            <![CDATA[
            #aacc00 {
                stroke: #000066;
                fill: #aacc00;
            }
            ]]>
        </style>

        <use href="#aacc00" x="10" fill="blue"/>
        <rect id="aacc00" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );

    test_optimize!(
        test_shorten_id_used_in_attribute_via_url,
        shorten_ids,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <defs>
            <linearGradient id="grad">
                <stop stop-color="#9a9582" offset="0"/>
                <stop stop-color="#adaa9f" offset="1"/>
            </linearGradient>
        </defs>

        <rect fill="url(#grad)" x="10" y="10" width="100" height="100"/>
        </svg>
        "##,
        r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 120 120">
        <defs>
            <linearGradient id="g">
                <stop stop-color="#9a9582" offset="0"/>
                <stop stop-color="#adaa9f" offset="1"/>
            </linearGradient>
        </defs>

        <rect fill="url(#g)" x="10" y="10" width="100" height="100"/>
        </svg>
        "##
    );
}
