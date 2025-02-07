import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { AttrFuncInputSpec } from "../bindings/AttrFuncInputSpec.ts";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { SocketSpecArity } from "../bindings/SocketSpecArity.ts";
import { SocketSpecKind } from "../bindings/SocketSpecKind.ts";
import { ExpandedPropSpec } from "./props.ts";
import { getSiFuncId } from "./siFuncs.ts";
import _ from "npm:lodash";
import { ExpandedSchemaVariantSpec } from "./pkgs.ts";

export const SI_SEPARATOR = "\u{b}";

export type ExpandedSocketSpec = SocketSpec & {
  data: NonNullable<SocketSpec["data"]>;
};
export function createOutputSocketFromProp(
  prop: ExpandedPropSpec,
  arity: SocketSpecArity = "many",
): ExpandedSocketSpec {
  const socket = createSocket(prop.name, "output", arity);
  socket.data.funcUniqueId = getSiFuncId("si:identity");
  socket.inputs = [attrFuncInputSpecFromProp(prop)];
  return socket;
}

export type ConnectionAnnotation = { tokens: string[] };

export function createInputSocketFromProp(
  prop: ExpandedPropSpec,
  arity: SocketSpecArity = "many",
  extraConnectionAnnotations?: ConnectionAnnotation[],
): ExpandedSocketSpec {
  const socket = createSocket(
    prop.name,
    "input",
    arity,
    extraConnectionAnnotations,
  );
  if (prop.kind === "array" && socket.data) {
    prop.data.inputs = [attrFuncInputSpecFromSocket(socket, "value")];
    prop.data.funcUniqueId = getSiFuncId("si:normalizeToArray");
  } else {
    prop.data.inputs = [attrFuncInputSpecFromSocket(socket)];
    prop.data.funcUniqueId = getSiFuncId("si:identity");
  }

  return socket;
}

export function getOrCreateInputSocketFromProp(
  schemaVariant: ExpandedSchemaVariantSpec,
  prop: ExpandedPropSpec,
  arity: SocketSpecArity = "many",
) {
  let socket = schemaVariant.sockets.find((s) =>
    s.data.kind === "input" && s.name === prop.name
  );
  if (!socket) {
    socket ??= createInputSocketFromProp(prop, arity);
    schemaVariant.sockets.push(socket);
  }
  return socket;
}

export function setAnnotationOnSocket(
  socket: ExpandedSocketSpec,
  annotation: ConnectionAnnotation,
) {
  const existingAnnotations = JSON.parse(
    socket.data.connectionAnnotations,
  ) as ConnectionAnnotation[];
  if (!existingAnnotations?.length) {
    throw new Error(`Bad connection annotations on ${socket.name}`);
  }

  let exists = false;
  for (const a of existingAnnotations) {
    if (_.isEqual(a, annotation)) {
      exists = true;
      break;
    }
  }

  if (!exists) {
    existingAnnotations.push(annotation);

    socket.data.connectionAnnotations = JSON.stringify(existingAnnotations);
  }
}

export function createSocket(
  name: string,
  kind: SocketSpecKind,
  arity: SocketSpecArity = "many",
  extraConnectionAnnotations: ConnectionAnnotation[] = [],
): ExpandedSocketSpec {
  const socketId = ulid();

  const data = {
    funcUniqueId: null,
    kind,
    name,
    connectionAnnotations: JSON.stringify([
      { "tokens": [name] },
      ...extraConnectionAnnotations,
    ]),
    arity,
    uiHidden: false,
  };

  const socket = {
    name,
    data,
    inputs: [],
    uniqueId: socketId,
  };

  return socket;
}

export function propPathToString(array: string[]): string {
  return array.join(SI_SEPARATOR);
}

export function attrFuncInputSpecFromProp(
  prop: ExpandedPropSpec,
  name: string = "identity",
): AttrFuncInputSpec {
  const prop_path = propPathToString(prop.metadata.propPath);
  const attr: AttrFuncInputSpec = {
    kind: "prop",
    name,
    prop_path,
    unique_id: ulid(),
    deleted: false,
  };

  return attr;
}

export function attrFuncInputSpecFromSocket(
  socket: SocketSpec,
  name: string = "identity",
): AttrFuncInputSpec {
  const attr: AttrFuncInputSpec = {
    kind: "inputSocket",
    name,
    socket_name: socket.name,
    unique_id: ulid(),
    deleted: false,
  };

  return attr;
}

export function propHasSocket(prop: ExpandedPropSpec): boolean {
  return prop.data.inputs?.find((i: AttrFuncInputSpec) =>
    i.kind === "inputSocket"
  ) !== undefined;
}
