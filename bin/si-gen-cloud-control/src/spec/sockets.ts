import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { AttrFuncInputSpec } from "../bindings/AttrFuncInputSpec.ts";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { SocketSpecArity } from "../bindings/SocketSpecArity.ts";
import { SocketSpecKind } from "../bindings/SocketSpecKind.ts";
import { ExpandedPropSpec } from "./props.ts";
import { getSiFuncId } from "./siFuncs.ts";

export const SI_SEPARATOR = "\u{b}";

export function createSocketFromProp(
  prop: ExpandedPropSpec,
): SocketSpec | null {
  if (prop.metadata.readOnly) {
    const socket = createSocket(prop.name, "output");
    if (socket.data) {
      socket.data.funcUniqueId = getSiFuncId("si:identity");
      socket.inputs.push(attrFuncInputSpecFromProp(prop));
    }
    return socket;
  } else if (prop.metadata.writeOnly || prop.metadata.createOnly) {
    const socket = createSocket(prop.name, "input");
    if (prop.data) {
      prop.data.inputs?.push(attrFuncInputSpecFromSocket(socket));
      prop.data.funcUniqueId = getSiFuncId("si:identity");
    }
    return socket;
  }

  return null;
}

function createSocket(name: string, kind: SocketSpecKind): SocketSpec {
  const socketId = ulid();

  const data = {
    funcUniqueId: null,
    kind,
    name,
    connectionAnnotations: JSON.stringify([{ "tokens": [name] }]),
    arity: "many" as SocketSpecArity,
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
): AttrFuncInputSpec {
  const attr: AttrFuncInputSpec = {
    kind: "inputSocket",
    name: "identity",
    socket_name: socket.name,
    unique_id: ulid(),
    deleted: false,
  };

  return attr;
}
