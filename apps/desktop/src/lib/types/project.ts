export interface CalculationSummary {
  primaryLabel: string;
  primaryValue: string;
  secondaryLabel?: string;
  secondaryValue?: string;
}

export interface CalculationRecord {
  id: string;
  methodId: string;
  methodName: string;
  label?: string;
  createdAt: string;
  input: unknown;
  result: unknown;
  summary: CalculationSummary;
}

export interface Scenario {
  id: string;
  name: string;
  calculationIds: string[];
}

export interface ProjectFile {
  version: number;
  name: string;
  createdAt: string;
  updatedAt: string;
  calculations: CalculationRecord[];
  scenarios: Scenario[];
}
