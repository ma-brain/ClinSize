//! DOCX export for calculation summaries.

use std::borrow::Cow;
use std::io::Cursor;

use docx::core::Core;
use docx::document::{Paragraph, Run, Text};
use docx::formatting::CharacterProperty;
use docx::Docx;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::error::{Error, Result};

const BODY_SIZE_HALF_POINTS: usize = 21;
const H1_SIZE_HALF_POINTS: usize = 40;
const H2_SIZE_HALF_POINTS: usize = 24;

fn owned(text: &str) -> Cow<'static, str> {
    Cow::Owned(text.to_owned())
}

fn text_run(text: &str, bold: bool, size: usize, color: Option<String>) -> Run<'static> {
    let mut property = CharacterProperty::default().size(size);
    if bold {
        property = property.bold(true);
    }
    if let Some(color) = color {
        property = property.color(color);
    }

    Run::default()
        .property(property)
        .push_text(Text::from(text.to_owned()))
}

struct DocxBuilder {
    docx: Docx<'static>,
    current_runs: Vec<Run<'static>>,
    heading: Option<HeadingLevel>,
    list_prefix: Option<&'static str>,
    bold: bool,
    body_size: usize,
}

impl DocxBuilder {
    fn new(title: &str) -> Self {
        let docx = Docx {
            core: Some(Core {
                title: Some(owned(title)),
                creator: Some(owned("ClinSize")),
                ..Default::default()
            }),
            ..Default::default()
        };

        Self {
            docx,
            current_runs: Vec::new(),
            heading: None,
            list_prefix: None,
            bold: false,
            body_size: BODY_SIZE_HALF_POINTS,
        }
    }

    fn push_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        let (size, color, bold) = match self.heading {
            Some(HeadingLevel::H1) => (H1_SIZE_HALF_POINTS, None, true),
            Some(HeadingLevel::H2) => (H2_SIZE_HALF_POINTS, Some("3B5BDB".to_owned()), true),
            _ => (self.body_size, None, self.bold),
        };

        self.current_runs.push(text_run(text, bold, size, color));
    }

    fn flush_paragraph(&mut self) {
        if self.current_runs.is_empty() && self.list_prefix.is_none() {
            self.heading = None;
            return;
        }

        let mut paragraph = Paragraph::default();
        if let Some(prefix) = self.list_prefix.take() {
            paragraph = paragraph.push(text_run(prefix, false, self.body_size, None));
        }

        for run in self.current_runs.drain(..) {
            paragraph = paragraph.push(run);
        }

        self.docx.document.push(paragraph);
        self.heading = None;
        self.bold = false;
    }

    fn ingest(&mut self, markdown: &str) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(markdown, options);

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    self.flush_paragraph();
                    self.heading = Some(level);
                }
                Event::End(TagEnd::Heading(_)) => self.flush_paragraph(),
                Event::Start(Tag::List(_)) => {}
                Event::End(TagEnd::List(_)) => self.flush_paragraph(),
                Event::Start(Tag::Item) => {
                    self.flush_paragraph();
                    self.list_prefix = Some("• ");
                }
                Event::End(TagEnd::Item) => self.flush_paragraph(),
                Event::Start(Tag::Paragraph) => self.flush_paragraph(),
                Event::End(TagEnd::Paragraph) => self.flush_paragraph(),
                Event::Start(Tag::Strong) => self.bold = true,
                Event::End(TagEnd::Strong) => self.bold = false,
                Event::Text(text) => self.push_text(&text),
                Event::Code(text) => self.push_text(&text),
                Event::SoftBreak | Event::HardBreak => self.push_text(" "),
                Event::Rule => self.flush_paragraph(),
                _ => {}
            }
        }

        self.flush_paragraph();
    }

    fn into_bytes(mut self) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());
        self.docx
            .write(&mut buffer)
            .map_err(|err| Error::Export(format!("{err:?}")))?;
        Ok(buffer.into_inner())
    }
}

/// Render a Markdown calculation summary as a native DOCX document.
pub fn markdown_to_docx_bytes(markdown: &str, title: &str) -> Result<Vec<u8>> {
    let mut builder = DocxBuilder::new(title);
    builder.ingest(markdown);
    builder.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_docx_zip_bytes() {
        let bytes = markdown_to_docx_bytes(
            "# ClinSize calculation summary\n\n## Method\n\n- **Method:** Two-sample t-test\n",
            "Two-sample t-test",
        )
        .expect("docx bytes");

        assert!(bytes.len() > 100);
        assert_eq!(&bytes[..2], b"PK");
    }
}
