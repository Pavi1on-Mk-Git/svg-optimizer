use crate::node::Node;

pub fn apply_to_nodes<I, F>(nodes: I, func: F) -> Vec<Node>
where
    I: IntoIterator<Item = Node>,
    F: Fn(Node) -> Option<Node>,
{
    nodes.into_iter().filter_map(func).collect()
}

macro_rules! use_optimization {
    ($fn_name:ident) => {
        mod $fn_name;
        pub use $fn_name::$fn_name;
    };
}

use_optimization!(ellipsis_to_circles);
use_optimization!(remove_comments);
use_optimization!(remove_useless_groups);
use_optimization!(shorten_ids);
use_optimization!(remove_attr_whitespace);

#[cfg(test)]
pub mod test {
    macro_rules! test_optimize {
        ($test_name:ident, $tested_fn:ident, $test_str:literal, $result:literal) => {
            #[test]
            fn $test_name() -> Result<(), ParserError> {
                let test_string = $test_str;

                let mut parser = Parser::new(test_string.as_bytes())?;

                let nodes = parser.parse_document()?;

                let nodes = $tested_fn(nodes);

                let buffer = Vec::new();
                let mut writer = SVGWriter::new(buffer);
                writer.write(nodes)?;

                let result = String::from_utf8(writer.into_inner()).unwrap();

                assert_eq!(result, $result);

                Ok(())
            }
        };
    }

    pub(crate) use test_optimize;
}
