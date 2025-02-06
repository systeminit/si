import { PkgSpec } from "../bindings/PkgSpec.ts";
import _ from "npm:lodash";
import {
  ConnectionAnnotation,
  propHasSocket,
  propPathToString,
} from "../spec/sockets.ts";
import {
  bfsExpandedPropTree,
  ExpandedPropSpec,
} from "../spec/props.ts";
import pluralize from "npm:pluralize";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import { getOrCreateInputSocketFromProp } from "../spec/sockets.ts";

export function createInputSocketsBasedOnOutputSockets(
  specs: PkgSpec[],
): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];
  const foundOutputSockets = {} as Record<string, SchemaVariantSpec[]>;
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
        foundOutputSockets[socket.name] ??= [];
        foundOutputSockets[socket.name].push(schemaVariant);

        // add annotations as we may generate relevant output socket annotations
        // that match props
        const existingAnnotations = JSON.parse(
          socket.data?.connectionAnnotations,
        ) as ConnectionAnnotation[];

        for (const annotations of existingAnnotations) {
          for (const annotation of annotations.tokens) {
            foundOutputSockets[annotation] ??= [];
            foundOutputSockets[annotation].push(schemaVariant);
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
    for (const prop of domain.entries as ExpandedPropSpec[]) {
      const fromVariants = foundOutputSockets[prop.name];
      if (!fromVariants) continue;
      // We don't create input sockets *just* to link to the same output socket/component.
      // There has to be another reason.
      if (fromVariants.length === 1 && fromVariants[0].uniqueId === schemaVariant.uniqueId) continue;
      getOrCreateInputSocketFromProp(schemaVariant, prop, "many");
    }

    // Create sockets for all Arns
    // TODO: we can be smarter about this, but this covers off on every case of
    // wanting to connecting something like "TaskArn" or "Arn" -> "TaskRoleArn"
    for (const prop of domain.entries as ExpandedPropSpec[]) {
      if (!prop.name.toLowerCase().endsWith("arn")) continue;
      getOrCreateInputSocketFromProp(schemaVariant, prop, "many", ["Arn"]);
    }

    // create input sockets for all strings and arrays of strings whose props name matches
    // the name of a component that exists
    bfsExpandedPropTree(domain, (prop) => {
      if (
          (!propHasSocket(prop) && (prop.kind === "array" && prop.typeProp.kind === "string"))
        || prop.kind === "string"
      ) {
        const possiblePeers = specsByName[prop.name] ??
          specsByName[pluralize(prop.name)];
        if (!possiblePeers) return;

        for (const peer of possiblePeers) {
          // If the peer has more than one primary identifier, we can't connect a single
          // output socket to it, so don't!
          let primaryIdentifierCount = 0;
          bfsExpandedPropTree([peer.domain, peer.resourceValue], (prop) => {
            if (prop.metadata.primaryIdentifier) {
              primaryIdentifierCount++;
            }
          })
          if (primaryIdentifierCount > 1) continue;

          bfsExpandedPropTree([peer.resourceValue, peer.domain], (peerProp) => {
            if (peerProp.metadata.primaryIdentifier) {
              for (const socket of peer.sockets) {
                const bind = socket.inputs[0];
                if (!bind) continue;
                if (
                  bind.kind === "prop" &&
                  bind.prop_path ===
                    propPathToString(peerProp.metadata.propPath)
                ) {
                  getOrCreateInputSocketFromProp(schemaVariant, prop, "many", [prop.name]);
                }
              }
            }
          }, {
            skipTypeProps: true,
          });
        }
      }
    }, { skipTypeProps: true });

    newSpecs.push(spec);
  }

  return newSpecs;
}
