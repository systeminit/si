import { existsSync } from "node:fs";
import _ from "lodash";

export type Inferred = {
  enum: string[] | null;
};

export async function loadInferred(
  inferredFile: string,
): Promise<Record<string, Inferred>> {
  if (!existsSync(inferredFile)) return {};
  return JSON.parse(await Deno.readTextFile(inferredFile));
}

export async function saveInferred(
  inferredFile: string,
  data: Record<string, Inferred>,
) {
  console.log(`Saving cache to ${inferredFile}...`);
  // Sort the output so it's easier to look at
  const sorted = Object.fromEntries(
    Object.entries(data).sort(([name1, value1], [name2, value2]) => {
      // 1. Values with enums first
      if (value1.enum !== null && value2.enum === null) return -1;
      if (value1.enum === null && value2.enum !== null) return 1;
      if (value1.enum !== null && value2.enum !== null) {
        // 2. Larger enums first
        if (value1.enum.length > value2.enum.length) return -1;
        if (value1.enum.length < value2.enum.length) return 1;
        // 3. Alphabetical order of the first enum value
        if (value1.enum[0] > value2.enum[0]) return -1;
        if (value1.enum[0] < value2.enum[0]) return 1;
      }
      // 4. Alphabetical order of description (mostly relevant for things w/o enums)
      if (name1 < name2) return -1;
      if (name1 > name2) return 1;
      return 0;
    }),
  );
  await Deno.writeTextFile(inferredFile, JSON.stringify(sorted, null, 2));
}
