pub(crate) mod constants;
pub(crate) mod id_generator;
pub(crate) mod id_usage;
pub(crate) mod iter;
pub(crate) mod replace_ids;
pub(crate) mod unit;

#[cfg(test)]
pub(crate) mod test {
    macro_rules! test_optimize {
        ($test_name:ident, $tested_fn:ident, $test_str:literal, $expected:literal) => {
            #[test]
            fn $test_name() -> anyhow::Result<()> {
                let mut parser = Parser::new($test_str.as_bytes())?;
                let nodes = parser.parse_document()?;

                let nodes = $tested_fn(nodes);

                let buffer = Vec::new();
                let mut writer = SVGWriter::new(buffer);
                writer.write(nodes)?;

                let actual = String::from_utf8(writer.into_inner()).unwrap();

                assert_eq!(actual, $expected.trim_end());

                Ok(())
            }
        };
    }

    pub(crate) use test_optimize;
}
