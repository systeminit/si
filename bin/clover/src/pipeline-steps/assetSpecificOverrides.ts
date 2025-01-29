import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";
import { ExpandedPropSpec } from "../spec/props.ts";
import _logger from "../logger.ts";
import { createInputSocketFromProp } from "../spec/sockets.ts";

const logger = _logger.ns("assetOverrides").seal();

export function assetSpecificOverrides(incomingSpecs: PkgSpec[]): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];

  for (const spec of incomingSpecs) {
    if (overrides.has(spec.name)) {
      logger.debug(`Running override for ${spec.name}`);
      overrides.get(spec.name)?.(spec);
    }
    newSpecs.push(spec);
  }

  return newSpecs;
}

type OverrideFn = (spec: PkgSpec) => void;

const overrides = new Map<string, OverrideFn>([
  ["AWS::EC2::Route", (spec: PkgSpec) => {
    addGatewayIdSocketToEC2Route(spec);
  }],
]);

function addGatewayIdSocketToEC2Route(spec: PkgSpec) {
  const schema = spec.schemas[0];
  const variant = spec.schemas[0].variants[0];
  const domain = variant.domain;

  if (!schema || !variant || !domain || domain.kind !== "object") {
    throw new Error(`Unable to run override for ${spec.name}`);
  }
  for (const prop of domain.entries) {
    if (prop.name === "GatewayId") {
      const socket = createInputSocketFromProp(prop as ExpandedPropSpec, "one");

      const data = socket.data;
      if (data) {
        const annotation = JSON.parse(data.connectionAnnotations);
        annotation.push({ tokens: ["InternetGatewayId"] });
        annotation.push({ tokens: ["VPNGatewayId"] });
        data.connectionAnnotations = JSON.stringify(annotation);
      }

      variant.sockets.push(socket);
    }
  }
}
