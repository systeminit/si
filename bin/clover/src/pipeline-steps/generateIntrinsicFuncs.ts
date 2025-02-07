import _ from "npm:lodash";
import {
  createNormalizeToArray,
  createResourcePayloadToValue,
  createSiFuncs,
} from "../spec/siFuncs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function generateIntrinsicFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const funcs = spec.funcs;

    spec.funcs = [
      ...funcs,
      ...createSiFuncs(),
      createResourcePayloadToValue(),
      createNormalizeToArray(),
    ];
    newSpecs.push(spec);
  }

  return newSpecs;
}
