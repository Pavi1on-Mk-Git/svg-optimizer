use crate::node::Node;
use anyhow::Result;
use itertools::Itertools;

mod common;

pub fn _apply_to_nodes_err<F>(nodes: Vec<Node>, func: F) -> Result<Vec<Node>>
where
    F: Fn(Node) -> Result<Option<Node>>,
{
    nodes
        .into_iter()
        .map(func)
        .process_results(|iter| iter.flatten().collect())
}

pub fn apply_to_nodes<I, F>(nodes: I, func: F) -> Vec<Node>
where
    I: IntoIterator<Item = Node>,
    F: Fn(Node) -> Option<Node>,
{
    nodes.into_iter().filter_map(func).collect()
}

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
        remove_useless_groups,
        no_remove_useless_groups,
        "Remove groups that contain a single node or no nodes.",
    ],
    [
        shorten_ids,
        no_shorten_ids,
        "Convert id names to be as short as possible. New names will only be created from latin alphabet letters and digits.",
    ],
    [
        remove_attr_whitespace,
        no_remove_attr_whitespace,
        "Remove excess whitespace from attributes.",
    ]
);

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    macro_rules! test_optimize {
        ($test_name:ident, $tested_fn:ident, $test_str:literal, $expected:literal) => {
            #[test]
            fn $test_name() -> anyhow::Result<()> {
                let test_string = $test_str.trim();

                let mut parser = Parser::new(test_string.as_bytes())?;
                let nodes = parser.parse_document()?;

                let nodes = $tested_fn(nodes)?;

                let buffer = Vec::new();
                let mut writer = SVGWriter::new(buffer);
                writer.write(nodes)?;

                let actual = String::from_utf8(writer.into_inner()).unwrap();

                assert_eq!(actual, $expected.trim());

                Ok(())
            }
        };
    }

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

    pub(crate) use test_optimize;
}
