import { PkgSpec } from "../bindings/PkgSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import _ from "lodash";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { bfsPropTree, isExpandedPropSpec } from "../spec/props.ts";
import {
  createOutputSocketFromProp,
  setAnnotationOnSocket,
} from "../spec/sockets.ts";

export function generateOutputSocketsFromProps(
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
      ...createSocketsFromPrimaryIdentifier(schemaVariant),
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
  variant: SchemaVariantSpec,
): SocketSpec[] {
  const domain = variant.domain;

  if (domain.kind !== "object") throw "Domain prop is not object";

  const sockets: SocketSpec[] = [];

  bfsPropTree(domain, (prop) => {
    if (!isExpandedPropSpec(prop)) return;

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
