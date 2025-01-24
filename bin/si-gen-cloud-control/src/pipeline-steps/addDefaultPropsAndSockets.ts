import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "lodash";
import {
  createObjectProp,
  createScalarProp,
  isExpandedPropSpec,
} from "../spec/props.ts";
import { getSiFuncId } from "../spec/siFuncs.ts";
import { attrFuncInputSpecFromSocket, createSocket } from "../spec/sockets.ts";

export function addDefaultPropsAndSockets(specs: PkgSpec[]): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];

  for (const spec of specs) {
    const schemaVariant = spec.schemas[0]?.variants[0];

    if (!schemaVariant) {
      console.log(
        `Could not generate extra props for ${spec.name}: missing schema or variant`,
      );
      continue;
    }

    const domain = schemaVariant.domain;
    if (!isExpandedPropSpec(domain)) {
      console.log(
        `Could not generate extra props for ${spec.name}: domain has no metadata`,
      );
      continue;
    }
    if (domain.kind !== "object") {
      console.log(
        `Could not generate extra props for ${spec.name}: domain is not object`,
      );
      continue;
    }

    // Region socket
    const regionSocket = createSocket("Region", "input", "one");
    schemaVariant.sockets.push(regionSocket);

    const extraProp = createObjectProp("extra", domain.metadata.propPath);

    const regionProp = createScalarProp(
      "Region",
      "string",
      extraProp.metadata.propPath,
    );
    if (regionProp.data) {
      regionProp.data.inputs = [
        attrFuncInputSpecFromSocket(regionSocket),
      ];
      regionProp.data.funcUniqueId = getSiFuncId("si:identity");
    }
    extraProp.entries.push(regionProp);
    domain.entries.push(extraProp);

    // Finalize
    newSpecs.push(spec);
  }

  return newSpecs;
}
