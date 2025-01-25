import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";
import {
  createNormalizeToArray,
  createResourcePayloadToValue,
  createSiFuncs,
} from "../spec/siFuncs.ts";

export function generateIntrinsicFuncs(specs: PkgSpec[]): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];

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
