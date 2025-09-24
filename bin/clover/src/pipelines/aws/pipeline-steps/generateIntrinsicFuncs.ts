import _ from "npm:lodash";
import { createSiFuncs } from "../../../spec/siFuncs.ts";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";

export function generateIntrinsicFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const funcs = spec.funcs;

    spec.funcs = [
      ...funcs,
      ...createSiFuncs(),
    ];
    newSpecs.push(spec);
  }

  return newSpecs;
}
