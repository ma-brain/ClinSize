/** One-way sensitivity analysis types (UI only; calculations stay in Rust). */

export interface SensitivityPoint {
  parameterValue: number;
  outputValue: number | null;
  error?: string;
}

export interface SensitivityOptionDef {
  id: string;
  label: string;
  getValues: () => number[];
  mutate: (input: unknown, value: number) => unknown;
}

/** Helper for typed method configs passed into the generic panel. */
export function asSensitivityOptions<TInput>(
  options: Array<{
    id: string;
    label: string;
    getValues: () => number[];
    mutate: (input: TInput, value: number) => TInput;
  }>,
): SensitivityOptionDef[] {
  return options.map((option) => ({
    id: option.id,
    label: option.label,
    getValues: option.getValues,
    mutate: (input, value) => option.mutate(input as TInput, value),
  }));
}
