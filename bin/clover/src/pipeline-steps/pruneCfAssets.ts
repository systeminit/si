import { ExpandedPkgSpecWithSockets } from "../spec/pkgs.ts";
import _logger from "../logger.ts";
import {
  CODE_GENERATION_FUNC_SPECS,
  createFunc,
  strippedBase64,
} from "../spec/funcs.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { FuncArgumentSpec } from "../bindings/FuncArgumentSpec.ts";
import { createSocket, propPathToString } from "../spec/sockets.ts";

const logger = _logger.ns("pruneCfAssets").seal();

export function pruneCfAssets(
  specs: readonly ExpandedPkgSpecWithSockets[],
) {
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [variant] = schema.variants;

    if (!spec.name.includes("::") || variant.cfSchema.handlers) {
      continue;
    }

    logger.debug(
      `Pruning ${schema.name} because it has no handlers`,
    );

    variant.sockets = variant.sockets.filter((socket) =>
      socket.data.kind === "input"
    );

    variant.managementFuncs = [];

    variant.leafFunctions = variant.leafFunctions.filter((func) =>
      func.funcUniqueId ===
        CODE_GENERATION_FUNC_SPECS.awsCloudFormationLint.id
    );

    const attrFunc = createAttributeFunc();
    spec.funcs.push(attrFunc);

    const socket = createCloudFormationResourceOutputSocket(attrFunc.uniqueId);
    variant.sockets.push(socket);
  }
}

function createAttributeFunc() {
  const code = Deno.readTextFileSync(
    "./src/cloud-control-funcs/attribute/awsCloudControlCfAssetAttr.ts",
  );
  const codeBase64: string = strippedBase64(code);
  const args: FuncArgumentSpec[] = [
    {
      name: "cfnType",
      kind: "string",
      elementKind: null,
      uniqueId: ulid(),
      deleted: false,
    },
    {
      name: "cfnProperties",
      kind: "object",
      elementKind: null,
      uniqueId: ulid(),
      deleted: false,
    },
    {
      name: "cfnLogicalResourceName",
      kind: "string",
      elementKind: null,
      uniqueId: ulid(),
      deleted: false,
    },
  ];

  return createFunc(
    "Set attributes for building assets in CloudFormation",
    "jsAttribute",
    "json",
    codeBase64,
    "4dbf74c51d38d38a9247a501fc49e6f8332addab4343c5e46d3453fee55cfb6a",
    args,
  );
}

function createCloudFormationResourceOutputSocket(
  funcUniqueId: string,
) {
  const name = "CloudFormation Resource";
  const socket = createSocket(
    name,
    "output",
    "many",
  );
  socket.data.funcUniqueId = funcUniqueId;
  socket.inputs = [
    {
      kind: "prop",
      name: "cfnType",
      prop_path: propPathToString([
        "root",
        "domain",
        "extra",
        "AwsResourceType",
      ]),
      unique_id: ulid(),
      deleted: false,
    },
    {
      kind: "prop",
      name: "cfnProperties",
      prop_path: propPathToString(["root", "domain"]),
      unique_id: ulid(),
      deleted: false,
    },
    {
      kind: "prop",
      name: "cfnLogicalResourceName",
      prop_path: propPathToString(["root", "si", "name"]),
      unique_id: ulid(),
      deleted: false,
    },
  ];

  return socket;
}
