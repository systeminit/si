import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { AttrFuncInputSpec } from "../bindings/AttrFuncInputSpec.ts";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { SocketSpecData } from "../bindings/SocketSpecData.ts";
import { SocketSpecArity } from "../bindings/SocketSpecArity.ts";
import { SocketSpecKind } from "../bindings/SocketSpecKind.ts";
import { ExpandedPropSpec } from "./props.ts";
import { getSiFuncId } from "./siFuncs.ts";
import _ from "npm:lodash";
import { ExpandedSchemaVariantSpec } from "./pkgs.ts";
import { Extend } from "../extend.ts";

export const SI_SEPARATOR = "\u{b}";

export type ExpandedSocketSpec = Extend<SocketSpec, {
  data: NonNullable<SocketSpecData>;
}>;

export function createOutputSocketFromProp(
  prop: ExpandedPropSpec,
  overrideName?: string,
): ExpandedSocketSpec {
  const socket = createSocketFromPropInner(
    prop,
    "output",
    "many",
    overrideName,
  );
  socket.data.funcUniqueId = getSiFuncId("si:identity");
  socket.inputs = [attrFuncInputSpecFromProp(prop)];

  return socket;
}

export type ConnectionAnnotation = { tokens: string[] };

export function createInputSocketFromProp(
  prop: ExpandedPropSpec,
  extraConnectionAnnotations?: ConnectionAnnotation[],
  overrideName?: string,
): ExpandedSocketSpec {
  const socket = createSocketFromPropInner(
    prop,
    "input",
    prop.kind === "array" ? "many" : "one",
    overrideName,
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

export function createSocketFromPropInner(
  prop: ExpandedPropSpec,
  kind: "input" | "output",
  arity: SocketSpecArity,
  overrideName?: string,
  extraConnectionAnnotations?: ConnectionAnnotation[],
) {
  const socketName = overrideName ?? socketNameFromProp(prop);
  extraConnectionAnnotations ??= [];

  if (!overrideName && socketName !== prop.name) {
    extraConnectionAnnotations.push({ tokens: [prop.name] });
    extraConnectionAnnotations.push({
      tokens: createExtendedAnnotationForProp([prop.name], prop),
    });
  }

  extraConnectionAnnotations.push({
    tokens: createExtendedAnnotationForProp([prop.name], prop),
  });

  return createSocket(
    socketName,
    kind,
    arity,
    extraConnectionAnnotations,
  );
}

export function socketNameFromProp(prop: ExpandedPropSpec) {
  const propPath = prop.metadata.propPath;
  let socketName = prop.name;
  // If we get a prop inside an object on domain, let's name its socket a bit better
  if (
    propPath.length > 3 && propPath[2] !== "extra"
  ) {
    // Remove any unnecessary identifiers so the socket name does not become enormous
    // Regex says "remove this token if on end of string"
    const propParentName = propPath.slice(2, -1).map((name) =>
      name
        .replace(/Configuration$/, "")
        .replace(/Config$/, "")
        .replace(/Specification$/, "")
        .replace(/Options$/, "")
        .replace(/Definition$/, "")
        .replace(/Settings$/, "")
        .replace(/Info$/, "")
        .replace(/Parameters$/, "")
        .replace(/Attributes$/, "")
        .replace(/Preference$/, "")
        .replace(/Details$/, "")
    ).join("");

    socketName = `${propParentName}${prop.name}`;
  }

  return socketName;
}

export function getSocketOnVariant(
  variant: ExpandedSchemaVariantSpec,
  name: string,
  kind: SocketSpecKind,
) {
  return variant.sockets.find((s) => s.name === name && s.data.kind === kind);
}

export function getOrCreateInputSocketFromProp(
  schemaVariant: ExpandedSchemaVariantSpec,
  prop: ExpandedPropSpec,
) {
  const socketName = socketNameFromProp(prop);
  let socket = getSocketOnVariant(schemaVariant, socketName, "input");

  if (!socket) {
    socket = createInputSocketFromProp(prop);
    schemaVariant.sockets.push(socket);
  }
  return socket;
}

export function getOrCreateOutputSocketFromProp(
  schemaVariant: ExpandedSchemaVariantSpec,
  prop: ExpandedPropSpec,
) {
  const socketName = socketNameFromProp(prop);
  let socket = getSocketOnVariant(schemaVariant, socketName, "output");

  if (!socket) {
    socket = createOutputSocketFromProp(prop);
    schemaVariant.sockets.push(socket);
  }
  return socket;
}

export function createExtendedAnnotationForProp(
  annotation: string[],
  prop: ExpandedPropSpec,
) {
  switch (prop.kind) {
    case "string":
    case "number":
    case "boolean":
    case "float":
    case "json":
      return [...annotation, prop.kind, "scalar"];
    case "array":
    case "map":
      return createExtendedAnnotationForProp(annotation, prop.typeProp);
    case "object":
      return [...annotation];
  }
}

export function setAnnotationOnSocket(
  socket: ExpandedSocketSpec,
  annotation: string | ConnectionAnnotation,
) {
  if (typeof annotation === "string") {
    annotation = { tokens: [annotation] };
  }
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
  arity: SocketSpecArity,
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

  socket.data.connectionAnnotations = dedupeAnnotations(
    socket.data.connectionAnnotations,
  );

  return socket;
}

function dedupeAnnotations(annotations: string) {
  const seen = new Set<string>();
  const annotationsParsed: ConnectionAnnotation[] = JSON.parse(annotations);
  return JSON.stringify(annotationsParsed.filter((entry) => {
    const key = JSON.stringify(entry.tokens);
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  }));
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

export function removeInputSockets(
  variant: ExpandedSchemaVariantSpec,
  socketNamesToRemove: string[],
): number {
  if (!variant.sockets || !socketNamesToRemove.length) return 0;

  const initialCount = variant.sockets.length;
  const normalizedNames = socketNamesToRemove.map((name) => name.toLowerCase());

  // Find and remove the sockets that match the criteria
  const socketsToRemove: number[] = [];

  // First identify which sockets to remove (by index)
  variant.sockets.forEach((socket, index) => {
    if (socket.data.kind === "input") {
      const socketNameLower = socket.name.toLowerCase();
      if (normalizedNames.some((name) => socketNameLower === name)) {
        socketsToRemove.push(index);
      }
    }
  });

  // Remove the sockets in reverse order to avoid index shifting issues
  for (let i = socketsToRemove.length - 1; i >= 0; i--) {
    variant.sockets.splice(socketsToRemove[i], 1);
  }

  return initialCount - variant.sockets.length;
}
