import { invoke } from "@tauri-apps/api/core";

export async function fetchCalculationRationale(
  methodId: string,
  input: unknown,
  result: unknown,
): Promise<string> {
  return invoke<string>("export_calculation_rationale", {
    methodId,
    inputJson: JSON.stringify(input),
    resultJson: JSON.stringify(result),
  });
}

export async function fetchProtocolText(
  methodId: string,
  input: unknown,
  result: unknown,
): Promise<string> {
  return invoke<string>("export_protocol_text", {
    methodId,
    inputJson: JSON.stringify(input),
    resultJson: JSON.stringify(result),
  });
}
