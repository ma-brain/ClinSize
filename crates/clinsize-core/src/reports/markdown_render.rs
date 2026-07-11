//! Shared Markdown rendering helpers for export formats.

use pulldown_cmark::{html, Options, Parser};

/// Convert Markdown to an HTML fragment (no document wrapper).
pub fn markdown_to_html_fragment(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_bold_and_lists() {
        let html = markdown_to_html_fragment("## Method\n\n- **Alpha:** 0.05\n");
        assert!(html.contains("<h2>Method</h2>"));
        assert!(html.contains("<strong>Alpha:</strong>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
    }
}
