import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";
import {
  createInputSocketFromProp,
  propHasSocket,
  propPathToString,
  setAnnotationOnSocket,
} from "../spec/sockets.ts";
import {
  bfsPropTree,
  ExpandedPropSpec,
  isExpandedPropSpec,
} from "../spec/props.ts";
import pluralize from "npm:pluralize";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";

export function createInputSocketsBasedOnOutputSockets(
  specs: PkgSpec[],
): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];
  const foundOutputSockets = new Set<string>();
  const specsByName = {} as Record<string, SchemaVariantSpec[]>;

  // Get all output sockets
  for (const spec of specs) {
    const schema = spec.schemas[0];

    if (!schema) {
      console.log(
        `Could not generate input sockets for ${spec.name}: missing schema`,
      );
      continue;
    }
    const schemaVariant = schema.variants[0];

    if (!schemaVariant) {
      console.log(
        `Could not generate input sockets for ${spec.name}: missing variant`,
      );
      continue;
    }

    for (const socket of schemaVariant.sockets) {
      if (socket.data?.kind === "output") {
        foundOutputSockets.add(socket.name);
      }
    }

    // Get Name
    let name;
    const tokens = spec.name.split("::");
    if (tokens.length > 1) {
      name = tokens.pop();
    } else {
      name = spec.name.split(" ").pop();
    }

    if (!name) continue;
    name = pluralize(name);

    const entries = specsByName[name] ?? [];
    entries.push(schemaVariant);
    specsByName[name] = entries;
  }

  for (const spec of specs) {
    const schema = spec.schemas[0];

    if (!schema) {
      console.log(
        `Could not generate input for ${spec.name}: missing schema`,
      );
      continue;
    }

    const schemaVariant = schema.variants[0];

    if (!schemaVariant) {
      console.log(
        `Could not generate input for ${spec.name}: missing variant`,
      );
      continue;
    }

    const domain = schema.variants[0].domain;

    if (domain?.kind !== "object") {
      console.log(
        `Could not generate input for ${spec.name}: missing domain`,
      );
      continue;
    }

    // Create sockets that props match exactly
    for (const prop of domain.entries) {
      if (foundOutputSockets.has(prop.name)) {
        let found = false;
        for (const socket of schemaVariant.sockets) {
          if (socket.name == prop.name) found = true;
        }
        if (!found) {
          schemaVariant.sockets.push(
            createInputSocketFromProp(prop as ExpandedPropSpec),
          );
        }
      }
    }

    // create input sockets for all arrays of strings whose props name matches
    // the name of a component that exists
    bfsPropTree(domain, (prop) => {
      if (
        isExpandedPropSpec(prop) && !propHasSocket(prop) &&
        prop.kind === "array" && prop.typeProp.kind === "string"
      ) {
        const possiblePeers = specsByName[prop.name];
        if (!possiblePeers) return;

        for (const peer of possiblePeers) {
          bfsPropTree(peer.resourceValue, (peerProp) => {
            if (!isExpandedPropSpec(peerProp)) return;

            if (peerProp.metadata.primaryIdentifier) {
              for (const socket of peer.sockets) {
                const bind = socket.inputs[0];
                if (!bind) continue;
                if (
                  bind.kind === "prop" &&
                  bind.prop_path ===
                    propPathToString(peerProp.metadata.propPath)
                ) {
                  setAnnotationOnSocket(socket, { tokens: [prop.name] });
                  schemaVariant.sockets.push(
                    createInputSocketFromProp(
                      prop as ExpandedPropSpec,
                    ),
                  );
                }
              }
            }
          }, { skipTypeProps: true });
        }
      }
    }, { skipTypeProps: true });

    newSpecs.push(spec);
  }

  return newSpecs;
}
