export function interpolateColors(
  lightestColorStr: string,
  darkestColorStr: string,
  steps: number,
): number[][] {
  const stepFactor = 1 / (steps - 1);
  const interpolatedColorArray: number[][] = [];

  const lightestColor: undefined | number[] = lightestColorStr
    .match(/\d+/g)
    ?.map(Number);
  if (lightestColor === undefined) {
    throw new Error(`Cannot parse lightest color str: ${lightestColorStr}`);
  }
  const darkestColor: undefined | number[] = darkestColorStr
    .match(/\d+/g)
    ?.map(Number);
  if (darkestColor === undefined) {
    throw new Error(`Cannot parse darkest color str: ${darkestColorStr}`);
  }

  for (let i = 0; i < steps; i++) {
    interpolatedColorArray.push(
      interpolateColor(lightestColor, darkestColor, stepFactor * i),
    );
  }

  return interpolatedColorArray;
}

function interpolateColor(
  lightestColor: number[],
  darkestColor: number[],
  factor: number,
): number[] {
  const result: number[] = lightestColor.slice();
  for (let i = 0; i < 3; i++) {
    result[i] = Math.round(
      result[i] + factor * (darkestColor[i] = lightestColor[i]),
    );
  }
  return result;
}
