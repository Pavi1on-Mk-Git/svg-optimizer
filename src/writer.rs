use std::io::Write;
use xml::{writer::Error, EmitterConfig, EventWriter};

use crate::node::Node;

pub struct SVGWriter<W: Write> {
    writer: EventWriter<W>,
}

impl<W: Write> SVGWriter<W> {
    pub fn new(target: W) -> Self {
        let mut config = EmitterConfig::new()
            .write_document_declaration(false)
            .pad_self_closing(false);

        config.perform_escaping = false;

        Self {
            writer: config.create_writer(target),
        }
    }

    pub fn write(&mut self, nodes: Vec<Node>) -> Result<(), Error> {
        nodes.into_iter().try_for_each(|node| {
            node.into_iter()
                .try_for_each(|event| self.writer.write(event.as_writer_event().unwrap()))
        })?;

        Ok(())
    }

    #[cfg(test)]
    pub fn into_inner(self) -> W {
        self.writer.into_inner()
    }
}
