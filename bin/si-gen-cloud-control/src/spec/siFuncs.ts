import { FuncSpec } from "../bindings/FuncSpec.ts";
import { FuncSpecData } from "../bindings/FuncSpecData.ts";
import { FuncSpecBackendKind } from "../bindings/FuncSpecBackendKind.ts";
import { FuncSpecBackendResponseType } from "../bindings/FuncSpecBackendResponseType.ts";
import { FuncArgumentSpec } from "../bindings/FuncArgumentSpec.ts";
import { FuncArgumentKind } from "../bindings/FuncArgumentKind.ts";

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
};

function createArgument(funcName: string, kind: string): FuncArgumentSpec[] {
  if (kind === "unset") {
    return [];
  }

  const arg: FuncArgumentSpec = {
    // For the identity function, use "identity" as the argument name
    name: funcName === "si:identity" ? "identity" : "value",
    // For identity and validation functions, use "any" as the kind
    kind: (funcName === "si:identity" || funcName === "si:validation")
      ? "any"
      : kind as FuncArgumentKind,
    elementKind: (kind === "array" || kind === "map") ? "any" : null,
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

  const data: FuncSpecData = {
    name,
    displayName: null,
    description: null,
    handler: "",
    codeBase64: "",
    backendKind: func.kind as FuncSpecBackendKind,
    responseType: func.kind as FuncSpecBackendResponseType,
    hidden: false,
    link: null,
  };

  return {
    name,
    uniqueId: func.id,
    data,
    deleted: false,
    isFromBuiltin: null,
    arguments: createArgument(name, func.kind),
  };
}

export function getSiFuncId(kind: string): string {
  return funcSpecs[kind].id;
}

// Si uses a version of base64 that removes the padding at the end for some reason
export function strippedBase64(code: string) {
  return btoa(
    code,
  ).replace(/=/g, "");
}
