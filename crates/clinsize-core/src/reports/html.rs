//! HTML wrappers for exported calculation summaries.

/// Wrap a Markdown calculation summary in a printable HTML document.
pub fn markdown_to_html_document(markdown: &str, title: &str) -> String {
    let body = markdown
        .lines()
        .map(render_markdown_line)
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>{title}</title>
  <style>
    body {{
      font-family: Georgia, "Times New Roman", serif;
      color: #1a1d21;
      margin: 2rem auto;
      max-width: 48rem;
      line-height: 1.5;
    }}
    h1, h2 {{ margin-top: 1.5rem; }}
    ul {{ padding-left: 1.25rem; }}
    li {{ margin: 0.2rem 0; }}
    @media print {{
      body {{ margin: 1in; }}
    }}
  </style>
</head>
<body>
{body}
</body>
</html>"#
    )
}

/// Word-compatible HTML export using the same summary content.
pub fn markdown_to_word_html_document(markdown: &str, title: &str) -> String {
    let body = markdown
        .lines()
        .map(render_markdown_line)
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html xmlns:o="urn:schemas-microsoft-com:office:office"
      xmlns:w="urn:schemas-microsoft-com:office:word"
      xmlns="http://www.w3.org/TR/REC-html40">
<head>
  <meta charset="utf-8" />
  <title>{title}</title>
  <!--[if gte mso 9]><xml><w:WordDocument><w:View>Print</w:View></w:WordDocument></xml><![endif]-->
  <style>
    body {{ font-family: Calibri, Arial, sans-serif; font-size: 11pt; }}
    h1 {{ font-size: 16pt; }}
    h2 {{ font-size: 13pt; }}
  </style>
</head>
<body>
{body}
</body>
</html>"#
    )
}

fn render_markdown_line(line: &str) -> String {
    if line.is_empty() {
        return String::new();
    }
    if let Some(rest) = line.strip_prefix("# ") {
        return format!("<h1>{rest}</h1>");
    }
    if let Some(rest) = line.strip_prefix("## ") {
        return format!("<h2>{rest}</h2>");
    }
    if let Some(rest) = line.strip_prefix("- ") {
        return format!("<li>{rest}</li>");
    }
    format!("<p>{line}</p>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_document_includes_title_and_heading() {
        let html = markdown_to_html_document("# ClinSize\n\n## Method\n", "Test");
        assert!(html.contains("<title>Test</title>"));
        assert!(html.contains("<h1>ClinSize</h1>"));
        assert!(html.contains("<h2>Method</h2>"));
    }
}
