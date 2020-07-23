import { uniqueNamesGenerator, colors, animals } from "unique-names-generator";

export function generateName(): string {
  return uniqueNamesGenerator({
    dictionaries: [colors, animals],
    separator: "-",
    length: 2,
  });
}
