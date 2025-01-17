import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { SocketSpecKind } from "../bindings/SocketSpecKind.ts";
import { ExpandedPropSpec } from "./props.ts";

export function createSocketFromProp(
  prop: ExpandedPropSpec,
): SocketSpec | null {
  if (prop.metadata.readOnly) {
    return createSocket(prop.name, "output");
  } else if (prop.metadata.writeOnly || prop.metadata.readOnly) {
    return createSocket(prop.name, "input");
  }

  return null;
}

function createSocket(name: string, kind: SocketSpecKind): SocketSpec {
  const socketId = ulid();

  const data = {
    funcUniqueId: "",
    kind,
    name,
    connectionAnnotations: JSON.stringify([name]),
    arity: "many",
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
