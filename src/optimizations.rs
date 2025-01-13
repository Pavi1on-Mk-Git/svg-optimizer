use crate::node::Node;

pub fn apply_to_nodes<I, F>(nodes: I, func: F) -> Vec<Node>
where
    I: IntoIterator<Item = Node>,
    F: Fn(Node) -> Option<Node>,
{
    nodes.into_iter().filter_map(func).collect()
}

macro_rules! use_optimizations {
    ($($fn_name:ident),*) => {
        $(
            mod $fn_name;
            pub use $fn_name::$fn_name;
        )*
    };
}

use_optimizations!(
    ellipsis_to_circles,
    remove_comments,
    remove_useless_groups,
    shorten_ids,
    remove_attr_whitespace
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

                let nodes = $tested_fn(nodes);

                let buffer = Vec::new();
                let mut writer = SVGWriter::new(buffer);
                writer.write(nodes)?;

                let actual = String::from_utf8(writer.into_inner()).unwrap();

                assert_eq!(actual, $expected.trim());

                Ok(())
            }
        };
    }

    fn identity(nodes: Vec<Node>) -> Vec<Node> {
        nodes
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
