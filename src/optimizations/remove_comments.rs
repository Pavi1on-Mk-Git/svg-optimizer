use super::apply_to_nodes;
use super::common::define_remove_childless_node;
use crate::node::ChildlessNodeType;
use crate::node::Node;
use anyhow::Result;

define_remove_childless_node!(remove_comments, Comment);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizations::test::test_optimize;
    use crate::parser::Parser;
    use crate::writer::SVGWriter;

    test_optimize!(
        test_remove_comments,
        remove_comments,
        r#"
        <!-- comment -->
        <svg xmlns="http://www.w3.org/2000/svg"><!-- comment --></svg>
        <!-- comment -->
        "#,
        r#"
        <svg xmlns="http://www.w3.org/2000/svg"/>
        "#
    );
}
