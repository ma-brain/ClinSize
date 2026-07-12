import { invoke } from "@tauri-apps/api/core";

export function calculateMethod<Input, Result>(
  methodId: string,
  input: Input,
): Promise<Result> {
  return invoke<Result>("calculate_method", { methodId, input });
}

export function exportMethodMarkdown<Input, Result>(
  methodId: string,
  input: Input,
  result: Result,
): Promise<string> {
  return invoke<string>("export_method_markdown", { methodId, input, result });
}
