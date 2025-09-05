import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import { bfsPropTree } from "../spec/props.ts";

export function prettifySocketNames(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const { variants: [variant] } = spec.schemas[0];

    bfsPropTree([variant.domain, variant.resourceValue], (prop) => {
      if (prop.data.inputs) {
        for (const input of prop.data.inputs) {
          if (input.kind !== "prop") {
            input.socket_name = toSpaceCase(input.socket_name);
          }
        }
      }
    });

    newSpecs.push(spec);
  }

  return newSpecs;
}

function toSpaceCase(name: string) {
  return name
    // separate any sequence of lowercase letters followed by an uppercase letter
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    // Separate any sequence of more than 3 of uppercase letters (acronyms) from the next word
    .replace(/([A-Z]{3,})([A-Z][a-z])/g, "$1 $2");
}
