import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";
import {
  ConnectionAnnotation,
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
import { SocketSpec } from "../bindings/SocketSpec.ts";

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

        // add annotations as we may generate relavant output socket annotations
        // that match props
        const existingAnnotations = JSON.parse(
          socket.data?.connectionAnnotations,
        ) as ConnectionAnnotation[];

        for (const annotations of existingAnnotations) {
          for (const annotation of annotations.tokens) {
            foundOutputSockets.add(annotation);
          }
        }
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
        if (!socketExistsInSockets(schemaVariant.sockets, prop.name)) {
          schemaVariant.sockets.push(
            createInputSocketFromProp(prop as ExpandedPropSpec),
          );
        }
      }
    }

    // Create sockets for all Arns
    // TODO: we can be smarter about this, but this covers off on every case of
    // wanting to connecting something like "TaskArn" or "Arn" -> "TaskRoleArn"
    for (const prop of domain.entries) {
      if (prop.name.toLowerCase().endsWith("arn")) {
        if (!socketExistsInSockets(schemaVariant.sockets, prop.name)) {
          const socket = createInputSocketFromProp(prop as ExpandedPropSpec);
          setAnnotationOnSocket(socket, { tokens: ["Arn"] });
          schemaVariant.sockets.push(socket);
        }
      }
    }

    // create input sockets for all strings and arrays of strings whose props name matches
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
                  if (
                    !socketExistsInSockets(
                      schemaVariant.sockets,
                      prop.name,
                    )
                  ) {
                    schemaVariant.sockets.push(
                      createInputSocketFromProp(
                        prop as ExpandedPropSpec,
                      ),
                    );
                  }
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

function socketExistsInSockets(
  sockets: SocketSpec[],
  name: string,
): boolean {
  for (const socket of sockets) {
    if (socket.name === name) return true;
  }
  return false;
}
