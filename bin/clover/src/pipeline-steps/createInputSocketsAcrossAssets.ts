import _ from "npm:lodash";

import {
  ConnectionAnnotation,
  createExtendedAnnotationForProp,
  createInputSocketFromProp,
  getSocketOnVariant,
  propPathToString,
  setAnnotationOnSocket,
  socketNameFromProp,
} from "../spec/sockets.ts";
import { bfsPropTree } from "../spec/props.ts";
import pluralize from "npm:pluralize";
import { getOrCreateInputSocketFromProp } from "../spec/sockets.ts";
import { ExpandedPkgSpecWithSockets, ExpandedSchemaVariantSpecWithSockets } from "../spec/pkgs.ts";

export function createInputSocketsBasedOnOutputSockets(
  specs: readonly ExpandedPkgSpecWithSockets[],
) {
  const foundOutputSockets = {} as Record<string, ExpandedSchemaVariantSpecWithSockets[]>;
  const specsByName = {} as Record<string, ExpandedSchemaVariantSpecWithSockets[]>;

  // Get all output sockets
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;

    for (const socket of schemaVariant.sockets) {
      if (socket.data?.kind === "output") {
        foundOutputSockets[socket.name.toLowerCase()] ??= [];
        foundOutputSockets[socket.name.toLowerCase()].push(schemaVariant);

        // add annotations as we may generate relevant output socket annotations
        // that match props
        const existingAnnotations = JSON.parse(
          socket.data?.connectionAnnotations,
        ) as ConnectionAnnotation[];

        for (const { tokens } of existingAnnotations) {
          const annotationToken = tokens[0];

          // One of the annotations is always the socket name. We'll skip that one
          if (annotationToken === socket.name) {
            continue;
          }

          foundOutputSockets[annotationToken.toLowerCase()] ??= [];
          foundOutputSockets[annotationToken.toLowerCase()].push(schemaVariant);
        }
      }
    }

    // Catalog assets by name
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
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const domain = schemaVariant.domain;

    // Create sockets that props match exactly
    for (const prop of domain.entries) {
      const fromVariants = foundOutputSockets[prop.name.toLowerCase()];
      if (!fromVariants) continue;
      // We don't create input sockets *just* to link to the same output socket/component.
      // There has to be another reason.
      if (
        fromVariants.length === 1 &&
        fromVariants[0].uniqueId === schemaVariant.uniqueId
      ) continue;
      getOrCreateInputSocketFromProp(schemaVariant, prop);
    }

    // Create sockets for all Arns
    // TODO: we can be smarter about this, but this covers off on every case of
    // wanting to connecting something like "TaskArn" or "Arn" -> "TaskRoleArn"
    bfsPropTree(domain, (prop) => {
      if (prop.name.toLowerCase().endsWith("arn")) {
        const socket = getOrCreateInputSocketFromProp(schemaVariant, prop);
        setAnnotationOnSocket(socket, { tokens: ["Arn"] });
        setAnnotationOnSocket(socket, {
          tokens: createExtendedAnnotationForProp(["arn"], prop),
        });
      }
    }, { skipTypeProps: true });

    // create input sockets for all strings and arrays of strings whose props name matches
    // the name of a component that exists
    bfsPropTree(domain, (prop) => {
      if (
        (
          prop.kind === "array" && prop.typeProp.kind === "string"
        ) ||
        prop.kind === "string"
      ) {
        const possiblePeers = specsByName[prop.name] ??
          specsByName[pluralize(prop.name)];
        if (!possiblePeers) return;

        for (const peer of possiblePeers) {
          // If the peer has more than one primary identifier, we can't connect a single
          // output socket to it, so don't!
          let primaryIdentifierCount = 0;
          bfsPropTree([peer.domain, peer.resourceValue], (prop) => {
            if (prop.metadata.primaryIdentifier) {
              primaryIdentifierCount++;
            }
          });
          if (primaryIdentifierCount > 1) continue;
          bfsPropTree([peer.resourceValue, peer.domain], (peerProp) => {
            if (!peerProp.metadata.primaryIdentifier) return;

            for (const socket of peer.sockets) {
              const bind = socket.inputs[0];
              if (!bind) continue;
              if (
                bind.kind === "prop" &&
                bind.prop_path ===
                  propPathToString(peerProp.metadata.propPath)
              ) {
                getOrCreateInputSocketFromProp(schemaVariant, prop);
                setAnnotationOnSocket(socket, { tokens: [prop.name] });
              }
            }
          }, {
            skipTypeProps: true,
          });
        }
      }
    }, { skipTypeProps: true });

    // for all arrays of scalars, create an input socket if one does not exist
    bfsPropTree(domain, (prop) => {
      if (
        prop.kind === "array" &&
        ["boolean", "string", "number", "float"].includes(prop.typeProp.kind)
      ) {
        const socketName = socketNameFromProp(prop);
        let socket = getSocketOnVariant(schemaVariant, socketName, "input");

        if (!socket) {
          socket = createInputSocketFromProp(prop);
          schemaVariant.sockets.push(socket);
          setAnnotationOnSocket(socket, {
            tokens: createExtendedAnnotationForProp([], prop),
          });
        }
      }
    }, { skipTypeProps: true });
  }
}
