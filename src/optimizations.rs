use crate::node::Node;
use anyhow::Result;

pub mod common;

macro_rules! use_optimizations {
    ($([$optimization_name:ident, $disable_flag_name:ident, $doc:literal,]),*) => {
        $(
            mod $optimization_name;
            pub use $optimization_name::$optimization_name;
        )*

        #[derive(clap::Parser)]
        pub struct Optimizations {
            $(
                #[arg(long)]
                #[doc = $doc]
                pub $optimization_name: bool,

                #[arg(long, conflicts_with = stringify!($optimization_name))]
                #[doc = "Disable the optimization."]
                pub $disable_flag_name: bool,
            )*
        }

        impl Optimizations {
            pub fn apply(&self, mut nodes: Vec<Node>, default_all: bool) -> Result<Vec<Node>> {
                $(
                    if self.$optimization_name || (default_all && !self.$disable_flag_name) {
                        nodes = $optimization_name(nodes)?;
                    }
                )*

                Ok(nodes)
            }
        }
    };
}

use_optimizations!(
    [
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
        remove_doctype,
        no_remove_doctype,
        "Remove DOCTYPE processing instruction.",
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
        "Extract common attributes in a group into the group",
    ]
);

#[cfg(test)]
pub mod test {
    use super::common::test::test_optimize;
    use crate::node::Node;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;
    use anyhow::Result;

    fn identity(nodes: Vec<Node>) -> Result<Vec<Node>> {
        Ok(nodes)
    }

    test_optimize!(
        test_no_optimizations,
        identity,
        r#"
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
