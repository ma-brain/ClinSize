/** TypeScript DTOs aligned with `clinsize-core` types. Update in the same change as Rust. */

export type Alternative = "two_sided" | "less" | "greater";

export type SolveMode = "sample_size" | "power" | "detectable_effect";

export interface CalculationWarning {
  code: string;
  message: string;
}

export type StudyObjective = "superiority" | "non_inferiority";

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

export interface OneWayAnovaInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  nPerGroup?: number;
  nGroups: number;
  betweenVariance: number;
  withinVariance: number;
  dropoutRate?: number;
}

export interface OneWayAnovaResult {
  nPerGroup: number;
  totalN: number;
  nPerGroupAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  effectSize: number;
  warnings: CalculationWarning[];
}

export interface AncovaTwoSampleInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  meanDifference: number;
  standardDeviation: number;
  baselineOutcomeCorrelation: number;
  allocationRatio: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface AncovaTwoSampleResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  effectSize: number;
  unadjustedStandardDeviation: number;
  adjustedStandardDeviation: number;
  baselineOutcomeCorrelation: number;
  varianceReductionFactor: number;
  warnings: CalculationWarning[];
}

export interface TwoProportionDifferenceInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  controlRate: number;
  treatmentRate: number;
  allocationRatio: number;
  alternative: Alternative;
  studyObjective: StudyObjective;
  noninferiorityMargin?: number;
  dropoutRate?: number;
}

export interface TwoProportionDifferenceResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  rateDifference: number;
  warnings: CalculationWarning[];
}

export interface OddsRatioInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  controlRate: number;
  treatmentRate: number;
  allocationRatio: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface OddsRatioResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  oddsRatio: number;
  warnings: CalculationWarning[];
}

export interface RiskRatioInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  controlRate: number;
  treatmentRate: number;
  allocationRatio: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface RiskRatioResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  riskRatio: number;
  warnings: CalculationWarning[];
}

export interface LogRankInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  totalEvents?: number;
  hazardRatio: number;
  allocationRatio: number;
  alternative: Alternative;
  controlHazardRate?: number;
  accrualDuration?: number;
  minimumFollowUp?: number;
  dropoutHazardRate?: number;
}

export interface LogRankResult {
  requiredEvents: number;
  eventsControl: number;
  eventsTreatment: number;
  achievedPower: number;
  hazardRatio: number;
  nControl?: number;
  nTreatment?: number;
  totalN?: number;
  probabilityEventControl?: number;
  probabilityEventTreatment?: number;
  warnings: CalculationWarning[];
}

export type MultiplicityMethod = "bonferroni" | "sidak" | "dunnett";

export interface MultiplicityInput {
  familyWiseAlpha: number;
  numberOfComparisons: number;
  adjustmentMethod: MultiplicityMethod;
}

export interface MultiplicityResult {
  adjustedAlpha: number;
  familyWiseAlpha: number;
  numberOfComparisons: number;
  adjustmentMethod: MultiplicityMethod;
  alphaReductionFactor: number;
  warnings: CalculationWarning[];
}
