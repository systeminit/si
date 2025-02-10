import _ from "lodash";
import { bfsPropTree } from "../spec/props.ts";
import {
  createOutputSocketFromProp,
  ExpandedSocketSpec,
  getSocketOnVariant,
  setAnnotationOnSocket,
} from "../spec/sockets.ts";
import { ExpandedPkgSpec, ExpandedSchemaVariantSpec } from "../spec/pkgs.ts";

export function generateOutputSocketsFromProps(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;

    schemaVariant.sockets = [
      ...schemaVariant.sockets,
      ...createSocketsFromResource(schemaVariant),
      ...createSocketsFromPrimaryIdentifier(schemaVariant),
      ...createSocketsForCommonProps(schemaVariant),
    ];

    newSpecs.push(spec);
  }

  return newSpecs;
}

function createSocketsFromResource(
  variant: ExpandedSchemaVariantSpec,
): ExpandedSocketSpec[] {
  const resource = variant.resourceValue;

  if (resource.kind !== "object") throw "Resource prop is not object";

  const sockets: ExpandedSocketSpec[] = [];
  for (const prop of resource.entries) {
    if (!["array", "object"].includes(prop.kind)) {
      const socket = createOutputSocketFromProp(prop);
      // if this socket is an arn, we want to make sure that all input sockets
      // that might also be arns can take this value
      if (socket.name.toLowerCase().endsWith("arn")) {
        const token = prop.name.slice(0, -3);
        if (token !== "") {
          setAnnotationOnSocket(socket, { tokens: [token] });
        }
      }
      sockets.push(socket);
    }
  }
  return sockets;
}

function createSocketsFromPrimaryIdentifier(
  variant: ExpandedSchemaVariantSpec,
): ExpandedSocketSpec[] {
  const domain = variant.domain;

  if (domain.kind !== "object") throw "Domain prop is not object";

  const sockets: ExpandedSocketSpec[] = [];

  bfsPropTree(domain, (prop) => {
    // We don't check if the socket already exists before adding, since on the other func
    // we only look at resourceValue props
    if (prop.metadata.primaryIdentifier) {
      sockets.push(createOutputSocketFromProp(prop));
    }
  }, {
    skipTypeProps: true,
  });

  return sockets;
}

// VariantName, VariantId props should always have sockets
function createSocketsForCommonProps(
  variant: ExpandedSchemaVariantSpec,
): ExpandedSocketSpec[] {
  const { domain } = variant;
  const variantName = variant.data.displayName;

  const sockets: ExpandedSocketSpec[] = [];

  bfsPropTree(domain, (prop) => {
    if (
      !["Name", "Id"].map((suffix) => `${variantName}${suffix}`).includes(
        prop.name,
      )
    ) return;

    // Don't duplicate sockets
    if (getSocketOnVariant(variant, prop.name, "output")) return;

    sockets.push(createOutputSocketFromProp(prop));
  }, {
    skipTypeProps: true,
  });

  return sockets;
}
