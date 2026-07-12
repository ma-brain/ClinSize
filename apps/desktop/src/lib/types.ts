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

export type AnovaEffect = "main_a" | "main_b" | "interaction";

export interface TwoWayAnovaInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  nPerCell?: number;
  nLevelsA: number;
  nLevelsB: number;
  primaryEffect: AnovaEffect;
  varianceA: number;
  varianceB: number;
  varianceInteraction: number;
  withinVariance: number;
  dropoutRate?: number;
}

export interface TwoWayAnovaResult {
  nPerCell: number;
  totalN: number;
  nPerCellAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  effectSize: number;
  primaryEffect: AnovaEffect;
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

export interface ChangeFromBaselineInput {
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

export interface ChangeFromBaselineResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  effectSize: number;
  unadjustedStandardDeviation: number;
  changeScoreStandardDeviation: number;
  baselineOutcomeCorrelation: number;
  warnings: CalculationWarning[];
}

export type CorrelationStructure =
  | "unstructured"
  | "ar1"
  | "compound_symmetry"
  | "toeplitz"
  | "csh";

export interface MmrmInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  treatmentEffect: number;
  residualStandardDeviation: number;
  correlationStructure: CorrelationStructure;
  correlation: number;
  nPostBaselineVisits: number;
  perVisitDropoutRate?: number;
  allocationRatio: number;
  alternative: Alternative;
}

export interface MmrmResult {
  /** Randomized counts; dropout is modeled inside the MMRM variance. */
  nControl: number;
  nTreatment: number;
  totalN: number;
  achievedPower: number;
  /** Lu-Luo-Chen variance factor φ (1 with complete data). */
  varianceFactor: number;
  finalRetention: number;
  cumulativeDropout: number;
  warnings: CalculationWarning[];
}

export interface NegativeBinomialInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  controlRate: number;
  treatmentRate: number;
  dispersion: number;
  exposureTime?: number;
  allocationRatio: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface NegativeBinomialResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  rateRatio: number;
  warnings: CalculationWarning[];
}

export interface PoissonInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  controlRate: number;
  treatmentRate: number;
  exposureTime?: number;
  allocationRatio: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface PoissonResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  rateRatio: number;
  warnings: CalculationWarning[];
}

export interface ProportionalOddsInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  controlN?: number;
  categoryProbabilities: number[];
  oddsRatio: number;
  treatmentFraction: number;
  dropoutRate?: number;
}

export interface ProportionalOddsResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  efficiency: number;
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

export interface OneSampleBinomialInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  n?: number;
  referenceRate: number;
  responseRate: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface OneSampleBinomialResult {
  n: number;
  nAdjusted: number;
  achievedPower: number;
  rateDifference: number;
  warnings: CalculationWarning[];
}

export interface MannWhitneyInput {
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

export interface MannWhitneyResult {
  nControl: number;
  nTreatment: number;
  totalN: number;
  nControlAdjusted: number;
  nTreatmentAdjusted: number;
  totalNAdjusted: number;
  achievedPower: number;
  probabilitySuperiority: number;
  effectSize: number;
  warnings: CalculationWarning[];
}

export interface WilcoxonSignedRankInput {
  solveMode: SolveMode;
  alpha: number;
  power?: number;
  nPairs?: number;
  meanDifference: number;
  standardDeviation: number;
  alternative: Alternative;
  dropoutRate?: number;
}

export interface WilcoxonSignedRankResult {
  nPairs: number;
  nPairsAdjusted: number;
  achievedPower: number;
  probabilityPositiveDifference: number;
  effectSize: number;
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

export type MultiplicityMethod =
  | "bonferroni"
  | "sidak"
  | "dunnett"
  | "holm"
  | "hochberg"
  | "graphical";

export interface MultiplicityInput {
  familyWiseAlpha: number;
  numberOfComparisons: number;
  adjustmentMethod: MultiplicityMethod;
  gatePosition?: number;
  comparisonWeights?: number[];
}

export interface MultiplicityResult {
  adjustedAlpha: number;
  familyWiseAlpha: number;
  numberOfComparisons: number;
  adjustmentMethod: MultiplicityMethod;
  gatePosition?: number;
  comparisonWeight?: number;
  alphaReductionFactor: number;
  warnings: CalculationWarning[];
}

export type SpendingFunction = "obrien_fleming" | "pocock";

export interface GroupSequentialInput {
  /** One-sided alpha spent on the upper efficacy boundary (0.025 ≙ two-sided 0.05). */
  alpha: number;
  targetPower: number;
  numberOfLooks: number;
  spendingFunction: SpendingFunction;
}

export interface GroupSequentialLookResult {
  look: number;
  informationFraction: number;
  incrementalAlphaSpent: number;
  cumulativeAlphaSpent: number;
  upperZBoundary: number;
}

export interface GroupSequentialResult {
  looks: GroupSequentialLookResult[];
  sampleSizeInflationFactor: number;
  requiredDrift: number;
  fixedDesignDrift: number;
  achievedPower: number;
  spendingFunction: SpendingFunction;
  warnings: CalculationWarning[];
}

export interface BlindedSsreInput {
  alpha: number;
  targetPower: number;
  meanDifference: number;
  plannedStandardDeviation: number;
  blindedInterimStandardDeviation?: number;
  interimFraction: number;
  allocationRatio: number;
  maxSampleSizeMultiplier: number;
  alternative: Alternative;
}

export interface BlindedSsreResult {
  plannedNControl: number;
  plannedNTreatment: number;
  plannedTotalN: number;
  interimNControl: number;
  interimNTreatment: number;
  interimTotalN: number;
  reEstimatedNControl: number;
  reEstimatedNTreatment: number;
  reEstimatedTotalN: number;
  cappedNControl: number;
  cappedNTreatment: number;
  cappedTotalN: number;
  sampleSizeInflationFactor: number;
  cappedInflationFactor: number;
  varianceRatio: number;
  /** Power at capped N under the planned SD (optimistic if variance rose). */
  achievedPowerAtCapped: number;
  /** Power at capped N under the blinded interim SD — the realistic estimate. */
  achievedPowerAtCappedInterimSd: number;
  wasCapped: boolean;
  warnings: CalculationWarning[];
}
