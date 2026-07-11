import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

export type ExportFormat = "markdown" | "html" | "word" | "pdf";

const EXTENSIONS: Record<ExportFormat, string> = {
  markdown: "md",
  html: "html",
  word: "doc",
  pdf: "html",
};

export async function exportCalculationSummary(options: {
  markdown: string;
  title: string;
  format: ExportFormat;
}) {
  const extension = EXTENSIONS[options.format];
  const path = await save({
    defaultPath: `clinsize-${slugify(options.title)}.${extension}`,
    filters: [{ name: options.format.toUpperCase(), extensions: [extension] }],
  });
  if (!path) return;

  let contents = options.markdown;
  if (options.format === "html" || options.format === "pdf") {
    contents = await invoke<string>("export_markdown_as_html", {
      markdown: options.markdown,
      title: options.title,
    });
  } else if (options.format === "word") {
    contents = await invoke<string>("export_markdown_as_word_html", {
      markdown: options.markdown,
      title: options.title,
    });
  }

  await writeTextFile(path, contents);
}

function slugify(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
}
