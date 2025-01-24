import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import _ from "lodash";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { isExpandedPropSpec } from "../spec/props.ts";
import { createSocketFromProp } from "../spec/sockets.ts";

export function generateSocketsFromDomainProps(specs: PkgSpec[]): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];

  for (const spec of specs) {
    const schemaVariant = spec.schemas[0]?.variants[0];

    if (!schemaVariant) {
      console.log(
        `Could not generate sockets for ${spec.name}: missing schema or variant`,
      );
      continue;
    }

    schemaVariant.sockets = [
      ...schemaVariant.sockets,
      ...createSocketsFromDomain(schemaVariant),
    ];

    newSpecs.push(spec);
  }

  return newSpecs;
}

function createSocketsFromDomain(variant: SchemaVariantSpec): SocketSpec[] {
  const domain = variant.domain;

  if (domain.kind !== "object") throw "Domain prop is not object";

  const sockets: SocketSpec[] = [];
  if (domain.kind == "object") {
    for (const prop of domain.entries) {
      if (
        !["array", "object"].includes(prop.kind) && isExpandedPropSpec(prop)
      ) {
        const socket = createSocketFromProp(prop);
        if (socket) {
          sockets.push(socket);
        }
      }
    }
  }
  return sockets;
}
