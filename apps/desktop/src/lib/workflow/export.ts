import { invoke } from "@tauri-apps/api/core";

export type ExportFormat = "markdown" | "html" | "word" | "pdf";

export async function exportCalculationSummary(options: {
  markdown: string;
  title: string;
  format: ExportFormat;
}) {
  let contents: Uint8Array;

  if (options.format === "pdf") {
    const pdfBytes = await invoke<number[]>("export_markdown_as_pdf", {
      markdown: options.markdown,
      title: options.title,
    });
    contents = new Uint8Array(pdfBytes);
  } else if (options.format === "word") {
    const docxBytes = await invoke<number[]>("export_markdown_as_docx", {
      markdown: options.markdown,
      title: options.title,
    });
    contents = new Uint8Array(docxBytes);
  } else {
    let text = options.markdown;
    if (options.format === "html") {
      text = await invoke<string>("export_markdown_as_html", {
        markdown: options.markdown,
        title: options.title,
      });
    }
    contents = new TextEncoder().encode(text);
  }

  await invoke<string | null>("save_export_file", {
    exportType: options.format,
    fileStem: `clinsize-${slugify(options.title)}`,
    contents: Array.from(contents),
  });
}

function slugify(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
}
