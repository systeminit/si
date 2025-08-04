import _ from "lodash";
import { bfsPropTree } from "../spec/props.ts";
import {
  createExtendedAnnotationForProp,
  setAnnotationOnSocket,
} from "../spec/sockets.ts";
import { ExpandedPkgSpecWithSockets } from "../spec/pkgs.ts";
import { socketNameFromProp } from "../spec/sockets.ts";
import { getSocketOnVariant } from "../spec/sockets.ts";

export function annotateCommonOutputSockets(
  specs: readonly ExpandedPkgSpecWithSockets[],
) {
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [variant] = schema.variants;
    const domain = variant.domain;
    const resource = variant.resourceValue;

    const category = schema.data.category.split("::")[1];
    const variantName = variant.data.displayName;

    bfsPropTree([resource, domain], (prop) => {
      if (prop.name.endsWith("Id")) {
        const propName = prop.name;
        const socketName = socketNameFromProp(prop);
        const socket = getSocketOnVariant(variant, socketName, "output");
        if (socket) {
          for (
            const annotation of [
              `${variantName}${propName}`,
              `${category}${variantName}${propName}`,
              `${category}${propName}`,
              `${variantName}${propName}entifier`,
              `${category}${propName}entifier`,
              `${variantName}${propName}entifier`,
              `${category}${variantName}${propName}entifer`,

              `${variantName} ${propName}`,
              `${category} ${variantName} ${propName}`,
              `${category} ${propName}`,
              `${variantName} ${propName}entifier`,
              `${category} ${propName}entifier`,
              `${variantName} ${propName}entifier`,
              `${category} ${variantName} ${propName}entifer`,
            ]
          ) {
            setAnnotationOnSocket(
              socket,
              {
                tokens: createExtendedAnnotationForProp([annotation], prop),
              },
            );
          }
        }
      }

      if (prop.name.endsWith("Name") || prop.metadata.primaryIdentifier) {
        const propName = prop.name;
        const socketName = socketNameFromProp(prop);
        const socket = getSocketOnVariant(variant, socketName, "output");
        if (socket) {
          for (
            const annotation of [
              `${variantName}${propName}`,
              `${category}${variantName}${propName}`,
              `${category}${propName}`,
              `${variantName} ${propName}`,
              `${category} ${propName}`,
              `${category} ${variantName} ${propName}`,
            ]
          ) {
            setAnnotationOnSocket(
              socket,
              {
                tokens: createExtendedAnnotationForProp([annotation], prop),
              },
            );
          }
        }
      }
    }, {
      skipTypeProps: true,
    });
  }
}
