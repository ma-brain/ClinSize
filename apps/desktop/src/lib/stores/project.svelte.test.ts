import { describe, it, expect } from "vitest";
import { summarizeResult } from "./project.svelte";

describe("summarizeResult", () => {
  describe("default branch (totalN methods)", () => {
    it("shows Total N and achieved power for two-sample t-test", () => {
      const summary = summarizeResult("continuous.two_sample_ttest", {
        totalN: 34,
        achievedPower: 0.807,
      });
      expect(summary.primaryLabel).toBe("Total N");
      expect(summary.primaryValue).toBe("34");
      expect(summary.secondaryLabel).toBe("Achieved power");
      expect(summary.secondaryValue).toBe("0.81");
    });

    it("falls back to totalNAdjusted when totalN is missing", () => {
      const summary = summarizeResult("continuous.mmrm", {
        totalNAdjusted: 44,
        achievedPower: 0.81,
      });
      expect(summary.primaryValue).toBe("44");
    });

    it("shows dash when neither totalN nor totalNAdjusted present", () => {
      const summary = summarizeResult("continuous.mmrm", { achievedPower: 0.81 });
      expect(summary.primaryValue).toBe("—");
    });
  });

  describe("one-sample methods (n field)", () => {
    it("shows N for one-sample t-test", () => {
      const summary = summarizeResult("continuous.one_sample_ttest", {
        n: 36,
        achievedPower: 0.802,
      });
      expect(summary.primaryLabel).toBe("N");
      expect(summary.primaryValue).toBe("36");
    });

    it("shows N for one-sample binomial", () => {
      const summary = summarizeResult("binary.one_sample_binomial", {
        n: 50,
        achievedPower: 0.9,
      });
      expect(summary.primaryLabel).toBe("N");
      expect(summary.primaryValue).toBe("50");
    });
  });

  describe("paired methods (nPairs field)", () => {
    it("shows Pairs for paired t-test", () => {
      const summary = summarizeResult("continuous.paired_ttest", {
        nPairs: 10,
        achievedPower: 0.803,
      });
      expect(summary.primaryLabel).toBe("Pairs");
      expect(summary.primaryValue).toBe("10");
    });

    it("shows Pairs for Wilcoxon signed-rank", () => {
      const summary = summarizeResult("continuous.wilcoxon_signed_rank", {
        nPairs: 131,
        achievedPower: 0.8,
      });
      expect(summary.primaryLabel).toBe("Pairs");
      expect(summary.primaryValue).toBe("131");
    });
  });

  describe("design methods", () => {
    it("shows Adjusted α for multiplicity", () => {
      const summary = summarizeResult("design.multiplicity", {
        adjustedAlpha: 0.025,
        numberOfComparisons: 2,
      });
      expect(summary.primaryLabel).toBe("Adjusted α");
      expect(summary.primaryValue).toBe("0.0250");
      expect(summary.secondaryLabel).toBe("Comparisons");
      expect(summary.secondaryValue).toBe("2");
    });

    it("shows Inflation factor and look count for group-sequential", () => {
      const summary = summarizeResult("design.group_sequential", {
        sampleSizeInflationFactor: 1.0128,
        looks: [{}, {}, {}],
      });
      expect(summary.primaryLabel).toBe("Inflation factor");
      expect(summary.primaryValue).toBe("1.01");
      expect(summary.secondaryLabel).toBe("Looks");
      expect(summary.secondaryValue).toBe("3");
    });

    it("shows dash for looks when not an array", () => {
      const summary = summarizeResult("design.group_sequential", {
        sampleSizeInflationFactor: 1.0,
      });
      expect(summary.secondaryValue).toBe("—");
    });

    it("shows Re-estimated N for blinded SSRE", () => {
      const summary = summarizeResult("design.blinded_ssre", {
        reEstimatedTotalN: 200,
        plannedTotalN: 180,
      });
      expect(summary.primaryLabel).toBe("Re-estimated N");
      expect(summary.primaryValue).toBe("200");
      expect(summary.secondaryLabel).toBe("Planned N");
      expect(summary.secondaryValue).toBe("180");
    });
  });

  describe("survival.log_rank", () => {
    it("shows Required events and Total enrolled", () => {
      const summary = summarizeResult("survival.log_rank", {
        requiredEvents: 128,
        totalN: 350,
      });
      expect(summary.primaryLabel).toBe("Required events");
      expect(summary.primaryValue).toBe("128");
      expect(summary.secondaryLabel).toBe("Total enrolled");
      expect(summary.secondaryValue).toBe("350");
    });

    it("omits secondary when totalN missing", () => {
      const summary = summarizeResult("survival.log_rank", {
        requiredEvents: 128,
      });
      expect(summary.secondaryLabel).toBeUndefined();
    });
  });

  describe("formatAlpha behavior (via summarizeResult)", () => {
    it("formats alpha floats to 4 decimal places", () => {
      const summary = summarizeResult("design.multiplicity", {
        adjustedAlpha: 0.026957839,
        numberOfComparisons: 5,
      });
      expect(summary.primaryValue).toBe("0.0270");
    });

    it("returns dash for missing numeric fields", () => {
      const summary = summarizeResult("design.multiplicity", {
        numberOfComparisons: 3,
      });
      expect(summary.primaryValue).toBe("—");
    });
  });
});
