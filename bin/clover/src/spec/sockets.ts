const SI_SEPARATOR = "\u{b}";

export function propPathToString(array: string[]): string {
  return array.join(SI_SEPARATOR);
}
