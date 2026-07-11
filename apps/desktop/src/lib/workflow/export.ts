import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile, writeTextFile } from "@tauri-apps/plugin-fs";

export type ExportFormat = "markdown" | "html" | "word" | "pdf";

const EXTENSIONS: Record<ExportFormat, string> = {
  markdown: "md",
  html: "html",
  word: "docx",
  pdf: "pdf",
};

const FILTER_NAMES: Record<ExportFormat, string> = {
  markdown: "Markdown",
  html: "HTML",
  word: "Word document",
  pdf: "PDF",
};

export async function exportCalculationSummary(options: {
  markdown: string;
  title: string;
  format: ExportFormat;
}) {
  const extension = EXTENSIONS[options.format];
  const path = await save({
    defaultPath: `clinsize-${slugify(options.title)}.${extension}`,
    filters: [{ name: FILTER_NAMES[options.format], extensions: [extension] }],
  });
  if (!path) return;

  if (options.format === "pdf") {
    const pdfBytes = await invoke<number[]>("export_markdown_as_pdf", {
      markdown: options.markdown,
      title: options.title,
    });
    await writeFile(path, new Uint8Array(pdfBytes));
    return;
  }

  if (options.format === "word") {
    const docxBytes = await invoke<number[]>("export_markdown_as_docx", {
      markdown: options.markdown,
      title: options.title,
    });
    await writeFile(path, new Uint8Array(docxBytes));
    return;
  }

  let contents = options.markdown;
  if (options.format === "html") {
    contents = await invoke<string>("export_markdown_as_html", {
      markdown: options.markdown,
      title: options.title,
    });
  }

  await writeTextFile(path, contents);
}

function slugify(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
}
