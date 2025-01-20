use crate::node::Node;

pub mod common;

macro_rules! use_optimizations {
    ([$([$regular_opt_name:ident, $disable_flag_name:ident, $regular_doc:literal,]),*],
    [$([$precision_opt_name:ident, $precision_doc:literal,]),*]) => {
        $(
            mod $regular_opt_name;
            use $regular_opt_name::$regular_opt_name;
        )*

        $(
            mod $precision_opt_name;
            use $precision_opt_name::$precision_opt_name;
        )*

        #[derive(clap::Parser)]
        pub struct Optimizations {
            $(
                #[arg(long)]
                #[doc = $regular_doc]
                $regular_opt_name: bool,

                #[arg(long, conflicts_with = stringify!($regular_opt_name))]
                #[doc = "Disable the optimization."]
                $disable_flag_name: bool,
            )*

            $(
                #[arg(long)]
                #[doc = $precision_doc]
                $precision_opt_name: bool,
            )*

            #[arg(short, long, default_value("3"))]
            #[doc = "Desired precision for lossy optimizations."]
            precision: usize,
        }

        impl Optimizations {
            pub fn apply(&self, mut nodes: Vec<Node>, default_all: bool) -> Vec<Node> {
                $(
                    if self.$regular_opt_name || (default_all && !self.$disable_flag_name) {
                        nodes = $regular_opt_name(nodes);
                    }
                )*

                $(
                    if self.$precision_opt_name {
                        nodes = $precision_opt_name(nodes, self.precision);
                    }
                )*

                nodes
            }

            $(
                #[cfg(test)]
                #[allow(dead_code)]
                pub fn $disable_flag_name(&self) -> bool {
                    self.$disable_flag_name
                }
            )*
        }
    };
}

use_optimizations!(
    [[
        remove_attribute_whitespace,
        no_remove_attribute_whitespace,
        "Remove excess whitespace from attributes.",
    ],
    [
        remove_whitespace_outside_tags,
        no_remove_whitespace_outside_tags,
        "Remove excess whitespace from outside of tags. Leaves whitespace between <text> tags, as it may be rendered.",
    ],
    [
        ellipses_to_circles,
        no_ellipses_to_circles,
        "Convert ellipses to circles if their `rx` and `ry` are equal.",
    ],
    [
        remove_comments,
        no_remove_comments,
        "Remove all comments.",
    ],
    [
        remove_descriptions,
        no_remove_descriptions,
        "Remove <title>, <metadata>, <desc> tags and their contents.",
    ],
    [
        remove_useless_groups,
        no_remove_useless_groups,
        "Remove groups that contain a single node or no nodes.",
    ],
    [
        remove_empty_attributes,
        no_remove_empty_attributes,
        "Remove attributes whose value is an empty string.",
    ],
    [
        remove_empty_texts,
        no_remove_empty_texts,
        "Remove empty <text>, <tspan>, <tref> tags.",
    ],
    [
        shorten_ids,
        no_shorten_ids,
        "Convert id names to be as short as possible. New names will only be created from latin alphabet letters and digits.",
    ],
    [
        remove_useless_ids,
        no_remove_useless_ids,
        "Removed unused ids.",
    ],
    [
        sort_attributes,
        no_sort_attributes,
        "Sorts attributes by name.",
    ],
    [
        extract_common_attributes,
        no_extract_common_attributes,
        "Extract common attributes in a group into the group.",
    ],
    [
        remove_unused_defs,
        no_remove_unused_defs,
        "Remove defined objects which are not used anywhere.",
    ],
    [
        remove_dimensions,
        no_convert_to_viewbox,
        "Remove width and height if they are equal to values in viewBox.",
    ],
    [
        remove_empty_containers,
        no_remove_empty_containers,
        "Remove empty container elements.",
    ],
    [
        remove_hidden_elements,
        no_remove_hidden_elements,
        "Remove elements which would not be rendered.",
    ],
    [
        remove_useless_stroke_and_fill,
        no_remove_useless_stroke_and_fill,
        "Remove fill and stroke attributes which would not be rendered.",
    ],
    [
        convert_paths_to_uses,
        no_convert_paths_to_uses,
        "Replace identical paths with <use>s of a single path.",
    ],
    [
        remove_editor_namespace_data,
        no_remove_editor_namespace_data,
        "Remove editor namespaces, elements and attributes.",
    ],
    [
        merge_consecutive_paths,
        no_merge_consecutive_paths,
        "Merge consecutive paths if their attributes match.",
    ]], [[
        round_floats,
        "Round floating point numbers to specified precision (disabled by default).",
    ], [
        merge_transforms,
        "Merge transform attribute components into one matrix (disabled by default).",
    ]]
);

#[cfg(test)]
pub mod test {
    use super::common::test::test_optimize;
    use crate::node::Node;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    fn identity(nodes: Vec<Node>) -> Vec<Node> {
        nodes
    }

    test_optimize!(
        test_no_optimizations,
        identity,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
        <svg xmlns="http://www.w3.org/2000/svg">
        <g fill="white" stroke="green" stroke-width="5"><circle cx="40" cy="40" r="25"/></g>
        <g><g/></g></svg>
        "#,
        r#"

        <svg xmlns="http://www.w3.org/2000/svg">
        <g fill="white" stroke="green" stroke-width="5"><circle cx="40" cy="40" r="25"/></g>
        <g><g/></g></svg>
        "#
    );
}
