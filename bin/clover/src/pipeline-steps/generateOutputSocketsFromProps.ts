import _ from "lodash";
import { bfsPropTree } from "../spec/props.ts";
import {
  getOrCreateOutputSocketFromProp,
  setAnnotationOnSocket,
} from "../spec/sockets.ts";
import { ExpandedPkgSpecWithSockets, ExpandedSchemaVariantSpecWithSockets } from "../spec/pkgs.ts";

export function generateOutputSocketsFromProps(
  specs: readonly ExpandedPkgSpecWithSockets[],
) {
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;

    // These funcs modify the variant directly
    createSocketsFromResource(schemaVariant);
    createSocketsFromPrimaryIdentifier(schemaVariant);
    createSocketsForCommonProps(schemaVariant);
  }
}

function createSocketsFromResource(
  variant: ExpandedSchemaVariantSpecWithSockets,
) {
  const resource = variant.resourceValue;

  if (resource.kind !== "object") throw "Resource prop is not object";

  for (const prop of resource.entries) {
    const socket = getOrCreateOutputSocketFromProp(variant, prop);
    // if this socket is an arn, we want to make sure that all input sockets
    // that might also be arns can take this value
    if (socket.name.toLowerCase().endsWith("arn")) {
      const token = prop.name.slice(0, -3);
      if (token !== "") {
        setAnnotationOnSocket(socket, { tokens: [token] });
      }
    }
  }
}

function createSocketsFromPrimaryIdentifier(
  variant: ExpandedSchemaVariantSpecWithSockets,
) {
  const domain = variant.domain;

  if (domain.kind !== "object") throw "Domain prop is not object";

  bfsPropTree(domain, (prop) => {
    // We don't check if the socket already exists before adding, since on the other func
    // we only look at resourceValue props
    if (prop.metadata.primaryIdentifier) {
      getOrCreateOutputSocketFromProp(variant, prop);
    }
  }, {
    skipTypeProps: true,
  });
}

// VariantName, VariantId props should always have sockets
function createSocketsForCommonProps(
  variant: ExpandedSchemaVariantSpecWithSockets,
) {
  const { domain } = variant;
  const variantName = variant.data.displayName;

  bfsPropTree(domain, (prop) => {
    if (
      !["Name", "Id"].map((suffix) => `${variantName}${suffix}`).includes(
        prop.name,
      )
    ) return;

    // Don't duplicate sockets
    getOrCreateOutputSocketFromProp(variant, prop);
  }, {
    skipTypeProps: true,
  });
}
