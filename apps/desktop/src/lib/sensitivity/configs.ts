import type {
  AncovaTwoSampleInput,
  OneSampleTTestInput,
  OneWayAnovaInput,
  PairedTTestInput,
  SolveMode,
  TwoSampleTTestInput,
} from "$lib/types";
import { centeredRange, linearRange, ratioRange } from "./ranges";
import type { SensitivityOptionDef } from "./types";
import { asSensitivityOptions } from "./types";

function parseOptionalRate(value: string): number {
  const trimmed = value.trim();
  return trimmed === "" ? 0 : Number(trimmed);
}

export function twoSampleSensitivityOptions(
  solveMode: SolveMode,
  meanDifference: string,
  standardDeviation: string,
  alpha: string,
  power: string,
  allocationRatio: string,
  dropoutRate: string,
): SensitivityOptionDef[] {
  const mean = Number(meanDifference);
  const sd = Number(standardDeviation);
  const alphaValue = Number(alpha);
  const powerValue = Number(power);
  const ratio = Number(allocationRatio);
  const dropout = parseOptionalRate(dropoutRate);

  const options = [
    {
      id: "meanDifference",
      label: "Mean difference",
      getValues: () => ratioRange(Math.abs(mean) || 0.5, 0.25, 2),
      mutate: (input: TwoSampleTTestInput, value: number) => ({
        ...input,
        meanDifference: value,
      }),
    },
    {
      id: "standardDeviation",
      label: "Standard deviation",
      getValues: () => ratioRange(sd || 1, 0.5, 2),
      mutate: (input: TwoSampleTTestInput, value: number) => ({
        ...input,
        standardDeviation: value,
      }),
    },
    {
      id: "alpha",
      label: "Type I error (alpha)",
      getValues: () => centeredRange(alphaValue || 0.05, 0.03, 0.01, 0.1, 9),
      mutate: (input: TwoSampleTTestInput, value: number) => ({ ...input, alpha: value }),
    },
    {
      id: "allocationRatio",
      label: "Allocation ratio",
      getValues: () => linearRange(0.5, 3, 11),
      mutate: (input: TwoSampleTTestInput, value: number) => ({
        ...input,
        allocationRatio: value,
      }),
    },
    {
      id: "dropoutRate",
      label: "Dropout rate",
      getValues: () => linearRange(0, Math.max(dropout, 0.4), 9),
      mutate: (input: TwoSampleTTestInput, value: number) => ({
        ...input,
        dropoutRate: value === 0 ? undefined : value,
      }),
    },
  ];

  if (solveMode === "sample_size") {
    options.splice(3, 0, {
      id: "power",
      label: "Target power",
      getValues: () => centeredRange(powerValue || 0.8, 0.15, 0.6, 0.95, 8),
      mutate: (input: TwoSampleTTestInput, value: number) => ({ ...input, power: value }),
    });
  }

  if (ratio > 0) {
    const ratioIndex = options.findIndex((option) => option.id === "allocationRatio");
    options[ratioIndex] = {
      ...options[ratioIndex],
      getValues: () => centeredRange(ratio, 1, 0.5, 3),
    };
  }

  return asSensitivityOptions(options);
}

function singleSampleOptions(
  solveMode: SolveMode,
  meanDifference: string,
  standardDeviation: string,
  alpha: string,
  power: string,
  dropoutRate: string,
) {
  const mean = Number(meanDifference);
  const sd = Number(standardDeviation);
  const alphaValue = Number(alpha);
  const powerValue = Number(power);
  const dropout = parseOptionalRate(dropoutRate);

  const options = [
    {
      id: "meanDifference",
      label: "Mean difference",
      getValues: () => ratioRange(Math.abs(mean) || 0.5, 0.25, 2),
      mutate: (input: OneSampleTTestInput, value: number) => ({
        ...input,
        meanDifference: value,
      }),
    },
    {
      id: "standardDeviation",
      label: "Standard deviation",
      getValues: () => ratioRange(sd || 1, 0.5, 2),
      mutate: (input: OneSampleTTestInput, value: number) => ({
        ...input,
        standardDeviation: value,
      }),
    },
    {
      id: "alpha",
      label: "Type I error (alpha)",
      getValues: () => centeredRange(alphaValue || 0.05, 0.03, 0.01, 0.1, 9),
      mutate: (input: OneSampleTTestInput, value: number) => ({ ...input, alpha: value }),
    },
    {
      id: "dropoutRate",
      label: "Dropout rate",
      getValues: () => linearRange(0, Math.max(dropout, 0.4), 9),
      mutate: (input: OneSampleTTestInput, value: number) => ({
        ...input,
        dropoutRate: value === 0 ? undefined : value,
      }),
    },
  ];

  if (solveMode === "sample_size") {
    options.splice(3, 0, {
      id: "power",
      label: "Target power",
      getValues: () => centeredRange(powerValue || 0.8, 0.15, 0.6, 0.95, 8),
      mutate: (input: OneSampleTTestInput, value: number) => ({ ...input, power: value }),
    });
  }

  return options;
}

export function oneSampleSensitivityOptions(
  solveMode: SolveMode,
  meanDifference: string,
  standardDeviation: string,
  alpha: string,
  power: string,
  dropoutRate: string,
): SensitivityOptionDef[] {
  return asSensitivityOptions(singleSampleOptions(
    solveMode,
    meanDifference,
    standardDeviation,
    alpha,
    power,
    dropoutRate,
  ));
}

export function pairedSensitivityOptions(
  solveMode: SolveMode,
  meanDifference: string,
  standardDeviation: string,
  alpha: string,
  power: string,
  dropoutRate: string,
): SensitivityOptionDef[] {
  return asSensitivityOptions(
    singleSampleOptions(
      solveMode,
      meanDifference,
      standardDeviation,
      alpha,
      power,
      dropoutRate,
    ).map((option) => ({
      ...option,
      mutate: (input: PairedTTestInput, value: number) =>
        option.mutate(input as unknown as OneSampleTTestInput, value) as PairedTTestInput,
    })),
  );
}

export function oneWayAnovaSensitivityOptions(
  solveMode: SolveMode,
  betweenVariance: string,
  withinSd: string,
  alpha: string,
  power: string,
  dropoutRate: string,
): SensitivityOptionDef[] {
  const between = Number(betweenVariance);
  const withinSdValue = Number(withinSd);
  const alphaValue = Number(alpha);
  const powerValue = Number(power);
  const dropout = parseOptionalRate(dropoutRate);

  const options = [
    {
      id: "betweenVariance",
      label: "Between-group variance",
      getValues: () => ratioRange(between || 1, 0.25, 2),
      mutate: (input: OneWayAnovaInput, value: number) => ({
        ...input,
        betweenVariance: value,
      }),
    },
    {
      id: "withinVariance",
      label: "Within-group SD",
      getValues: () => ratioRange(withinSdValue || 1, 0.5, 2),
      mutate: (input: OneWayAnovaInput, value: number) => ({
        ...input,
        withinVariance: value ** 2,
      }),
    },
    {
      id: "alpha",
      label: "Type I error (alpha)",
      getValues: () => centeredRange(alphaValue || 0.05, 0.03, 0.01, 0.1, 9),
      mutate: (input: OneWayAnovaInput, value: number) => ({ ...input, alpha: value }),
    },
    {
      id: "dropoutRate",
      label: "Dropout rate",
      getValues: () => linearRange(0, Math.max(dropout, 0.4), 9),
      mutate: (input: OneWayAnovaInput, value: number) => ({
        ...input,
        dropoutRate: value === 0 ? undefined : value,
      }),
    },
  ];

  if (solveMode === "sample_size") {
    options.splice(2, 0, {
      id: "power",
      label: "Target power",
      getValues: () => centeredRange(powerValue || 0.8, 0.15, 0.6, 0.95, 8),
      mutate: (input: OneWayAnovaInput, value: number) => ({ ...input, power: value }),
    });
  }

  return asSensitivityOptions(options);
}

export function ancovaSensitivityOptions(
  solveMode: SolveMode,
  meanDifference: string,
  standardDeviation: string,
  baselineOutcomeCorrelation: string,
  alpha: string,
  power: string,
  allocationRatio: string,
  dropoutRate: string,
): SensitivityOptionDef[] {
  const mean = Number(meanDifference);
  const sd = Number(standardDeviation);
  const correlation = Number(baselineOutcomeCorrelation);
  const alphaValue = Number(alpha);
  const powerValue = Number(power);
  const ratio = Number(allocationRatio);
  const dropout = parseOptionalRate(dropoutRate);

  const options = [
    {
      id: "meanDifference",
      label: "Mean difference",
      getValues: () => ratioRange(Math.abs(mean) || 0.5, 0.25, 2),
      mutate: (input: AncovaTwoSampleInput, value: number) => ({
        ...input,
        meanDifference: value,
      }),
    },
    {
      id: "standardDeviation",
      label: "Unadjusted standard deviation",
      getValues: () => ratioRange(sd || 1, 0.5, 2),
      mutate: (input: AncovaTwoSampleInput, value: number) => ({
        ...input,
        standardDeviation: value,
      }),
    },
    {
      id: "baselineOutcomeCorrelation",
      label: "Baseline-outcome correlation",
      getValues: () => centeredRange(correlation || 0, 0.4, -0.9, 0.9, 11),
      mutate: (input: AncovaTwoSampleInput, value: number) => ({
        ...input,
        baselineOutcomeCorrelation: value,
      }),
    },
    {
      id: "alpha",
      label: "Type I error (alpha)",
      getValues: () => centeredRange(alphaValue || 0.05, 0.03, 0.01, 0.1, 9),
      mutate: (input: AncovaTwoSampleInput, value: number) => ({ ...input, alpha: value }),
    },
    {
      id: "allocationRatio",
      label: "Allocation ratio",
      getValues: () => (ratio > 0 ? centeredRange(ratio, 1, 0.5, 3) : linearRange(0.5, 3, 11)),
      mutate: (input: AncovaTwoSampleInput, value: number) => ({
        ...input,
        allocationRatio: value,
      }),
    },
    {
      id: "dropoutRate",
      label: "Dropout rate",
      getValues: () => linearRange(0, Math.max(dropout, 0.4), 9),
      mutate: (input: AncovaTwoSampleInput, value: number) => ({
        ...input,
        dropoutRate: value === 0 ? undefined : value,
      }),
    },
  ];

  if (solveMode === "sample_size") {
    options.splice(4, 0, {
      id: "power",
      label: "Target power",
      getValues: () => centeredRange(powerValue || 0.8, 0.15, 0.6, 0.95, 8),
      mutate: (input: AncovaTwoSampleInput, value: number) => ({ ...input, power: value }),
    });
  }

  return asSensitivityOptions(options);
}
