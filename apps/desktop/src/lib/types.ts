/** TypeScript DTOs aligned with `clinsize-core` types. Update in the same change as Rust. */

export type Alternative = "two_sided" | "less" | "greater";

export type SolveMode = "sample_size" | "power" | "detectable_effect";

export interface CalculationWarning {
  code: string;
  message: string;
}

export interface MethodDescriptor {
  id: string;
  displayName: string;
  endpointCategory: string;
  supportedSolveModes: SolveMode[];
  documentationPath: string | null;
}
