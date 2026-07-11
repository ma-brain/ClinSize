import type { CalculationRecord, CalculationSummary, ProjectFile, Scenario } from "$lib/types/project";

function timestamp(): string {
  return String(Math.floor(Date.now() / 1000));
}

function newId(prefix: string): string {
  return `${prefix}-${Date.now()}`;
}

export function createProject(name: string): ProjectFile {
  const now = timestamp();
  return {
    version: 1,
    name,
    createdAt: now,
    updatedAt: now,
    calculations: [],
    scenarios: [],
  };
}

export const projectState = $state({
  project: createProject("Untitled project"),
  filePath: null as string | null,
  dirty: false,
});

export function touchProject() {
  projectState.project.updatedAt = timestamp();
  projectState.dirty = true;
}

export function recordCalculation(record: Omit<CalculationRecord, "id" | "createdAt">) {
  const entry: CalculationRecord = {
    ...record,
    id: newId("calc"),
    createdAt: timestamp(),
  };
  projectState.project.calculations.unshift(entry);
  touchProject();
  return entry;
}

export function removeCalculation(id: string) {
  projectState.project.calculations = projectState.project.calculations.filter(
    (entry) => entry.id !== id,
  );
  for (const scenario of projectState.project.scenarios) {
    scenario.calculationIds = scenario.calculationIds.filter((calcId) => calcId !== id);
  }
  touchProject();
}

export function renameCalculation(id: string, label: string) {
  const entry = projectState.project.calculations.find((item) => item.id === id);
  if (!entry) return;
  entry.label = label;
  touchProject();
}

export function createScenario(name: string, calculationIds: string[]): Scenario {
  const scenario: Scenario = {
    id: newId("scenario"),
    name,
    calculationIds,
  };
  projectState.project.scenarios.push(scenario);
  touchProject();
  return scenario;
}

export function removeScenario(id: string) {
  projectState.project.scenarios = projectState.project.scenarios.filter(
    (scenario) => scenario.id !== id,
  );
  touchProject();
}

export function setProject(project: ProjectFile, filePath: string | null = null) {
  projectState.project = project;
  projectState.filePath = filePath;
  projectState.dirty = false;
}

export function summarizeResult(methodId: string, result: Record<string, unknown>): CalculationSummary {
  switch (methodId) {
    case "survival.log_rank":
      return {
        primaryLabel: "Required events",
        primaryValue: String(result.requiredEvents ?? "—"),
        secondaryLabel: result.totalN ? "Total enrolled" : undefined,
        secondaryValue: result.totalN ? String(result.totalN) : undefined,
      };
    case "binary.two_proportion_difference":
      return {
        primaryLabel: "Total N",
        primaryValue: String(result.totalN ?? "—"),
        secondaryLabel: "Achieved power",
        secondaryValue: formatNumber(result.achievedPower),
      };
    default:
      return {
        primaryLabel: "Total N",
        primaryValue: String(result.totalN ?? result.totalNAdjusted ?? "—"),
        secondaryLabel: "Achieved power",
        secondaryValue: formatNumber(result.achievedPower),
      };
  }
}

function formatNumber(value: unknown): string {
  return typeof value === "number" ? value.toFixed(4) : "—";
}
