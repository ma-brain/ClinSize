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

export interface TwoSampleTTestInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  meanDifference: number;
  standardDeviation: number;
  allocationRatio: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface TwoSampleTTestResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  effectSize: number;
  warnings: CalculationWarning[];
}

export interface OneSampleTTestInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  n?: number;
  meanDifference: number;
  standardDeviation: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface OneSampleTTestResult {
  n: number;
  nAdjusted: number;
  achievedPower: number;
  effectSize: number;
  warnings: CalculationWarning[];
}

export interface PairedTTestInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  nPairs?: number;
  meanDifference: number;
  standardDeviation: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface PairedTTestResult {
  nPairs: number;
  nPairsAdjusted: number;
  achievedPower: number;
  effectSize: number;
  warnings: CalculationWarning[];
}
