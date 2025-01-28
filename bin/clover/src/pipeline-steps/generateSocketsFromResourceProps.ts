import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import _ from "lodash";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { isExpandedPropSpec } from "../spec/props.ts";
import { createOutputSocketFromProp } from "../spec/sockets.ts";

export function generateOutputSocketsFromResourceProps(
  specs: PkgSpec[],
): PkgSpec[] {
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
      ...createSocketsFromResource(schemaVariant),
    ];

    newSpecs.push(spec);
  }

  return newSpecs;
}

function createSocketsFromResource(variant: SchemaVariantSpec): SocketSpec[] {
  const resource = variant.resourceValue;

  if (resource.kind !== "object") throw "Resource prop is not object";

  const sockets: SocketSpec[] = [];
  for (const prop of resource.entries) {
    if (
      !["array", "object"].includes(prop.kind) && isExpandedPropSpec(prop)
    ) {
      sockets.push(createOutputSocketFromProp(prop));
    }
  }
  return sockets;
}
