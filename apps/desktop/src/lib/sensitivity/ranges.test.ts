import { describe, it, expect } from "vitest";
import { linearRange, ratioRange, centeredRange } from "./ranges";

describe("linearRange", () => {
  it("produces evenly spaced endpoints inclusive", () => {
    const result = linearRange(0, 10, 11);
    expect(result).toHaveLength(11);
    expect(result[0]).toBe(0);
    expect(result[10]).toBe(10);
    expect(result[5]).toBeCloseTo(5);
  });

  it("returns [min, max] for points=2", () => {
    expect(linearRange(3, 7, 2)).toEqual([3, 7]);
  });

  it("returns [min] when points <= 1", () => {
    expect(linearRange(5, 10, 1)).toEqual([5]);
    expect(linearRange(5, 10, 0)).toEqual([5]);
  });

  it("uses default 11 points", () => {
    expect(linearRange(0, 1)).toHaveLength(11);
  });

  it("handles descending ranges", () => {
    const result = linearRange(10, 0, 3);
    expect(result).toEqual([10, 5, 0]);
  });
});

describe("ratioRange", () => {
  it("scales center by min/max ratios", () => {
    const result = ratioRange(10, 0.5, 2.0, 3);
    expect(result[0]).toBe(5);
    expect(result[2]).toBe(20);
  });

  it("falls back to raw ratios when center <= 0", () => {
    const result = ratioRange(0, 0.5, 2.0, 3);
    expect(result[0]).toBe(0.5);
    expect(result[2]).toBe(2.0);
  });

  it("falls back to raw ratios when center is negative", () => {
    const result = ratioRange(-1, 0.5, 2.0, 3);
    expect(result[0]).toBe(0.5);
    expect(result[2]).toBe(2.0);
  });
});

describe("centeredRange", () => {
  it("sweeps symmetrically around center", () => {
    const result = centeredRange(0.8, 0.15, 0.6, 0.95, 5);
    expect(result[0]).toBeCloseTo(0.65);
    expect(result[4]).toBeCloseTo(0.95);
    expect(result).toHaveLength(5);
  });

  it("clamps low end to min", () => {
    const result = centeredRange(0.05, 0.1, 0.01, 0.1, 5);
    expect(result[0]).toBe(0.01);
  });

  it("clamps high end to max", () => {
    const result = centeredRange(0.9, 0.3, 0.01, 0.95, 5);
    expect(result[result.length - 1]).toBe(0.95);
  });

  it("returns [center] when window collapses (low >= high)", () => {
    // center=0.5, spread=0.01, min=0.4, max=0.45 → low=0.49, high=0.45 → collapsed
    const result = centeredRange(0.5, 0.01, 0.4, 0.45, 5);
    expect(result).toEqual([0.5]);
  });

  it("returns [center] when spread is zero and center within bounds", () => {
    const result = centeredRange(0.5, 0, 0.4, 0.6, 5);
    expect(result).toEqual([0.5]);
  });
});
