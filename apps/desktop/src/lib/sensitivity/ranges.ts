const DEFAULT_POINTS = 11;

/** Evenly spaced values from min to max (inclusive). */
export function linearRange(min: number, max: number, points = DEFAULT_POINTS): number[] {
  if (points <= 1) return [min];
  const step = (max - min) / (points - 1);
  return Array.from({ length: points }, (_, index) => min + step * index);
}

/** Scale a positive center value by min/max ratios. */
export function ratioRange(
  center: number,
  minRatio: number,
  maxRatio: number,
  points = DEFAULT_POINTS,
): number[] {
  if (center <= 0) {
    return linearRange(minRatio, maxRatio, points);
  }
  return linearRange(center * minRatio, center * maxRatio, points);
}

/** Symmetric sweep around a center, clamped to bounds. */
export function centeredRange(
  center: number,
  spread: number,
  min: number,
  max: number,
  points = DEFAULT_POINTS,
): number[] {
  const low = Math.max(min, center - spread);
  const high = Math.min(max, center + spread);
  if (low >= high) return [center];
  return linearRange(low, high, points);
}
