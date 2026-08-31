import { describe, it, expect } from "vitest";
import {
  twoSampleSensitivityOptions,
  negativeBinomialSensitivityOptions,
  twoWayAnovaSensitivityOptions,
} from "./configs";

describe("twoSampleSensitivityOptions", () => {
  it("produces expected number of options in sample-size mode", () => {
    const options = twoSampleSensitivityOptions(
      "sample_size",
      "1", "1", "0.05", "0.8", "1", "",
    );
    // controlRate, treatmentRate, meanDifference, alpha, power, allocationRatio, dropoutRate
    // = 7 base options + power spliced in for sample_size = 7 (power is already in the list)
    expect(options.length).toBeGreaterThanOrEqual(5);
    expect(options.some((o) => o.id === "alpha")).toBe(true);
  });

  it("mutate updates the correct field", () => {
    const options = twoSampleSensitivityOptions(
      "sample_size", "1", "1", "0.05", "0.8", "1", "",
    );
    const alphaOption = options.find((o) => o.id === "alpha");
    expect(alphaOption).toBeDefined();
    const baseInput = {
      solveMode: "sample_size" as const,
      alpha: 0.05,
      power: 0.8,
      meanDifference: 1,
      standardDeviation: 1,
      allocationRatio: 1,
      alternative: "two_sided" as const,
    };
    const mutated = alphaOption!.mutate(baseInput, 0.01) as { alpha: number };
    expect(mutated.alpha).toBe(0.01);
  });
});

describe("negativeBinomialSensitivityOptions", () => {
  it("includes dispersion sweep option", () => {
    const options = negativeBinomialSensitivityOptions(
      "sample_size", "2", "1", "1", "1", "0.05", "0.8", "1", "",
    );
    const dispersion = options.find((o) => o.id === "dispersion");
    expect(dispersion).toBeDefined();
  });

  it("produces values for each option", () => {
    const options = negativeBinomialSensitivityOptions(
      "sample_size", "2", "1", "1", "1", "0.05", "0.8", "1", "",
    );
    for (const opt of options) {
      const values = opt.getValues();
      expect(values.length).toBeGreaterThan(0);
    }
  });
});

describe("twoWayAnovaSensitivityOptions", () => {
  it("produces options including variance sweeps", () => {
    const options = twoWayAnovaSensitivityOptions(
      "sample_size", "0.5", "0.5", "0.5", "1", "0.05", "0.8", "",
    );
    expect(options.some((o) => o.id === "varianceA")).toBe(true);
    expect(options.some((o) => o.id === "varianceB")).toBe(true);
    expect(options.some((o) => o.id === "withinVariance")).toBe(true);
  });
});
