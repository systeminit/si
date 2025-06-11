import { FuncSpec } from "../bindings/FuncSpec.ts";
import { FuncArgumentSpec } from "../bindings/FuncArgumentSpec.ts";
import { FuncArgumentKind } from "../bindings/FuncArgumentKind.ts";
import { createFunc } from "./funcs.ts";
import { FuncSpecBackendKind } from "../bindings/FuncSpecBackendKind.ts";
import { FuncSpecBackendResponseType } from "../bindings/FuncSpecBackendResponseType.ts";

interface FuncSpecInfo {
  id: string;
  kind: string;
}

const funcSpecs: Record<string, FuncSpecInfo> = {
  "si:identity": {
    id: "c6938e12287ab65f8ba8234559178413f2e2c02c44ea08384ed6687a36ec4f50",
    kind: "identity",
  },
  "si:setArray": {
    id: "51049a590fb64860f159972012ac2657c629479a244d6bcc4b1b73ba4b29f87f",
    kind: "array",
  },
  "si:setBoolean": {
    id: "577a7deea25cfad0d4b2dd1e1f3d96b86b8b1578605137b8c4128d644c86964b",
    kind: "boolean",
  },
  "si:setInteger": {
    id: "7d384b237852f20b8dec2fbd2e644ffc6bde901d7dc937bd77f50a0d57e642a9",
    kind: "integer",
  },
  "si:setJson": {
    id: "c48ahif4739799f3ab84bcb88495f93b27b47c31a341f8005a60ca39308909fd",
    kind: "json",
  },
  "si:setMap": {
    id: "dea5084fbf6e7fe8328ac725852b96f4b5869b14d0fe9dd63a285fa876772496",
    kind: "map",
  },
  "si:setObject": {
    id: "cb9bf94739799f3a8b84bcb88495f93b27b47c31a341f8005a60ca39308909fd",
    kind: "object",
  },
  "si:setString": {
    id: "bbe86d1a2b92c3e34b72a407cca424878d3466d29ca60e56a251a52a0840bfbd",
    kind: "string",
  },
  "si:unset": {
    id: "8143ff98fbe8954bb3ab89ee521335d45ba9a42b7b79289eff53b503c4392c37",
    kind: "unset",
  },
  "si:validation": {
    id: "039ff70bc7922338978ab52a39156992b7d8e3390f0ef7e99d5b6ffd43141d8a",
    kind: "validation",
  },
  "si:normalizeToArray": {
    id: "750b9044cd250a5f0e952dabe4150fa61450992e04e688be47096d50a4759d4f",
    kind: "object",
  },
  "si:resourcePayloadToValue": {
    id: "bc58dae4f4e1361840ec8f081350d7ec6b177ee8dc5a6a55155767c92efe1850",
    kind: "object",
  },
  "si:setFloat": {
    id: "ab9875b8d5987e3f41e9d5a3c2cc00896338d89b084ca570fa22202c8da0ec55",
    kind: "float",
  },
};

function createArgument(funcName: string, kind: string): FuncArgumentSpec[] {
  if (kind === "unset") {
    return [];
  }

  const arg: FuncArgumentSpec = {
    // For the identity function, use "identity" as the argument name
    name: funcName === "si:identity" ? "identity" : "value",
    // For identity and validation functions, use "any" as the kind
    kind: funcName === "si:identity" || funcName === "si:validation"
      ? "any"
      : (kind as FuncArgumentKind),
    elementKind: kind === "array" || kind === "map" ? "any" : null,
    uniqueId: null,
    deleted: false,
  };

  return [arg];
}
export function createSiFunc(name: string): FuncSpec {
  const func = funcSpecs[name];
  if (!func) {
    throw new Error(`Unknown function: ${name}`);
  }

  return createFunc(
    name,
    func.kind as FuncSpecBackendKind,
    func.kind as FuncSpecBackendResponseType,
    "",
    func.id,
    createArgument(name, func.kind),
  );
}

export function getSiFuncId(kind: string): string {
  return funcSpecs[kind].id;
}

export function createSiFuncs(): FuncSpec[] {
  const ret: FuncSpec[] = [];
  const siFuncs = [
    "si:identity",
    "si:setArray",
    "si:setBoolean",
    "si:setInteger",
    "si:setJson",
    "si:setMap",
    "si:setObject",
    "si:setString",
    "si:unset",
    "si:validation",
    "si:setFloat",
    "si:normalizeToArray",
    "si:resourcePayloadToValue",
  ];

  for (const func of siFuncs) {
    ret.push(createSiFunc(func));
  }

  return ret;
}
