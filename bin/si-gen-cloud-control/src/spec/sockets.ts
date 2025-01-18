import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { SocketSpec } from "../bindings/SocketSpec.ts";
import { SocketSpecKind } from "../bindings/SocketSpecKind.ts";
import { ExpandedPropSpec } from "./props.ts";
import { SiPropFuncSpec } from "../bindings/SiPropFuncSpec.ts";
import { AttrFuncInputSpec } from "../bindings/AttrFuncInputSpec.ts";

// This is the unique id from lib/dal/src/func/intrinsics.rs
const IDENTITY_FUNC_UNIQUE_ID =
  "c6938e12287ab65f8ba8234559178413f2e2c02c44ea08384ed6687a36ec4f50";

export function createSocketFromProp(
  prop: ExpandedPropSpec,
): [SocketSpec, SiPropFuncSpec] | null {
  let socketKind: SocketSpecKind | undefined;
  if (prop.metadata.readOnly) {
    socketKind = "output";
  } else if (prop.metadata.writeOnly || prop.metadata.createOnly) {
    socketKind = "input";
  }

  if (!socketKind) {
    return null;
  }

  return createSocket(prop.name, socketKind, prop.metadata.path);
}

function createSocket(
  name: string,
  kind: SocketSpecKind,
  prop_path: string,
): [SocketSpec, SiPropFuncSpec] {
  const socketId = ulid();

  const data = {
    funcUniqueId: IDENTITY_FUNC_UNIQUE_ID,
    kind,
    name,
    connectionAnnotations: JSON.stringify([name]),
    arity: "many",
    uiHidden: false,
  };

  const inputSpec: AttrFuncInputSpec = {
    deleted: false,
    kind: "prop",
    name: "identity",
    prop_path,
    unique_id: null,
  };

  // FIXME Actually socket inputs seem to be empty on the real object. I don't get it
  const socket = {
    name,
    data,
    inputs: [inputSpec],
    uniqueId: socketId,
  };

  const func: SiPropFuncSpec = {
    kind: "name",
    funcUniqueId: "",
    uniqueId: null,
    deleted: false,
    inputs: [inputSpec],
  };

  return [socket, func];
}
