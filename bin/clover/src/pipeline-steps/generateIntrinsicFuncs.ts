import _ from "npm:lodash";
import { createSiFuncs } from "../spec/siFuncs.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function generateIntrinsicFuncs(
  specs: readonly ExpandedPkgSpec[],
) {
  for (const spec of specs) {
    const funcs = spec.funcs;

    spec.funcs = [
      ...funcs,
      ...createSiFuncs(),
    ];
  }
}
