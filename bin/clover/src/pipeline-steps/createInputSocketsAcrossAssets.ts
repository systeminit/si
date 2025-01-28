import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "lodash";
import { createInputSocketFromProp } from "../spec/sockets.ts";
import { ExpandedPropSpec } from "../spec/props.ts";

export function createInputSocketsBasedOnOutputSockets(
  specs: PkgSpec[],
): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];
  const foundOutputSockets = new Set<string>();

  for (const spec of specs) {
    const schema = spec.schemas[0];

    if (!schema) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing schema`,
      );
      continue;
    }
    const schemaVariant = schema.variants[0];

    if (!schemaVariant) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing variant`,
      );
      continue;
    }

    for (const socket of schemaVariant.sockets) {
      if (socket.data?.kind === "output") {
        foundOutputSockets.add(socket.name);
      }
    }
  }

  for (const spec of specs) {
    const schema = spec.schemas[0];

    if (!schema) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing schema`,
      );
      continue;
    }

    const schemaVariant = schema.variants[0];

    if (!schemaVariant) {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing variant`,
      );
      continue;
    }

    const domain = schema.variants[0].domain;

    if (domain?.kind !== "object") {
      console.log(
        `Could not generate default props and sockets for ${spec.name}: missing domain`,
      );
      continue;
    }

    for (const prop of domain.entries) {
      if (foundOutputSockets.has(prop.name)) {
        schemaVariant.sockets.push(
          createInputSocketFromProp(prop as ExpandedPropSpec),
        );
      }
    }

    newSpecs.push(spec);
  }

  return newSpecs;
}
