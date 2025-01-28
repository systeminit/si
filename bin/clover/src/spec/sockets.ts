import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { AttrFuncInputSpec } from "../bindings/AttrFuncInputSpec.ts";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { SocketSpecArity } from "../bindings/SocketSpecArity.ts";
import { SocketSpecKind } from "../bindings/SocketSpecKind.ts";
import { ExpandedPropSpec } from "./props.ts";
import { getSiFuncId } from "./siFuncs.ts";

export const SI_SEPARATOR = "\u{b}";

export function createOutputSocketFromProp(
  prop: ExpandedPropSpec,
  arity: SocketSpecArity = "many",
): SocketSpec {
  const socket = createSocket(prop.name, "output", arity);
  if (socket.data) {
    socket.data.funcUniqueId = getSiFuncId("si:identity");
    socket.inputs.push(attrFuncInputSpecFromProp(prop));
  }
  return socket;
}

export function createInputSocketFromProp(
  prop: ExpandedPropSpec,
  arity: SocketSpecArity = "many",
): SocketSpec {
  const socket = createSocket(prop.name, "input", arity);
  if (socket.data) {
    prop.data.inputs?.push(attrFuncInputSpecFromSocket(socket));
    prop.data.funcUniqueId = getSiFuncId("si:identity");
  }
  return socket;
}

export function createSocket(
  name: string,
  kind: SocketSpecKind,
  arity: SocketSpecArity = "many",
): SocketSpec {
  const socketId = ulid();

  const data = {
    funcUniqueId: null,
    kind,
    name,
    connectionAnnotations: JSON.stringify([{ "tokens": [name] }]),
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

export function attrFuncInputSpecFromProp(
  prop: ExpandedPropSpec,
): AttrFuncInputSpec {
  const prop_path = prop.metadata.propPath.join(SI_SEPARATOR);
  const attr: AttrFuncInputSpec = {
    kind: "prop",
    name: "identity",
    prop_path,
    unique_id: ulid(),
    deleted: false,
  };

  return attr;
}

export function attrFuncInputSpecFromSocket(
  socket: SocketSpec,
  name = "identity",
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
