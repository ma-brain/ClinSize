import { recordCalculation, summarizeResult } from "$lib/stores/project.svelte";

export function persistCalculation(options: {
  methodId: string;
  methodName: string;
  input: unknown;
  result: unknown;
}) {
  recordCalculation({
    methodId: options.methodId,
    methodName: options.methodName,
    input: options.input,
    result: options.result,
    summary: summarizeResult(
      options.methodId,
      options.result as Record<string, unknown>,
    ),
  });
}
