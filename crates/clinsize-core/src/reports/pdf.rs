//! PDF export for calculation summaries.

use crate::error::{Error, Result};
use markdown2pdf::config::ConfigSource;

const CLINSIZE_PDF_THEME: &str = r##"
theme = "academic"

[defaults]
text_color = "#14171C"
font_size_pt = 10.5
line_height = 1.55
text_align = "left"

[page]
size = "A4"
margins = { top = 22, right = 22, bottom = 26, left = 22 }

[headings.h1]
font_size_pt = 20.0
text_align = "left"
text_color = "#14171C"
margin_before_pt = 0.0
margin_after_pt = 10.0

[headings.h2]
font_size_pt = 12.0
font_weight = "bold"
text_color = "#3B5BDB"
margin_before_pt = 14.0
margin_after_pt = 6.0

[paragraph]
margin_after_pt = 2.0

[list.common]
item_spacing_loose_pt = 3.0
bullet_gap_pt = 6.0

[horizontal_rule]
color = "#D9DEE6"
margin_before_pt = 8.0
margin_after_pt = 8.0

[metadata]
creator = "ClinSize"
producer = "ClinSize"

[footer]
center = "ClinSize - Page {page} of {total_pages}"
"##;

/// Render a Markdown calculation summary as PDF bytes.
pub fn markdown_to_pdf_bytes(markdown: &str, title: &str) -> Result<Vec<u8>> {
    let document = format!("---\ntitle: {title}\n---\n\n{markdown}");

    markdown2pdf::parse_into_bytes(document, ConfigSource::Embedded(CLINSIZE_PDF_THEME), None)
        .map_err(|err| Error::Export(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_pdf_bytes_with_pdf_header() {
        let bytes = markdown_to_pdf_bytes(
            "# ClinSize calculation summary\n\n## Method\n\n- **Method:** Two-sample t-test\n",
            "Two-sample t-test",
        )
        .expect("pdf bytes");

        assert!(bytes.len() > 100);
        assert_eq!(&bytes[..4], b"%PDF");
    }
}
