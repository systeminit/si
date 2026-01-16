import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import _logger from "../../../logger.ts";
import { createFunc, strippedBase64 } from "../../../spec/funcs.ts";
import { CODE_GENERATION_FUNC_SPECS } from "../funcs.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { FuncArgumentSpec } from "../../../bindings/FuncArgumentSpec.ts";
import {
  createScalarProp,
  ExpandedPropSpecFor,
  findPropByName,
} from "../../../spec/props.ts";

const logger = _logger.ns("pruneCfAssets").seal();

export function pruneCfAssets(specs: ExpandedPkgSpec[]): ExpandedPkgSpec[] {
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [variant] = schema.variants;

    if (!spec.name.includes("::") || variant.superSchema.handlers?.read) {
      continue;
    }

    logger.debug(`Pruning ${schema.name} because it has no read handler`);

    variant.managementFuncs = [];

    variant.leafFunctions = variant.leafFunctions.filter(
      (func) =>
        func.funcUniqueId ===
          CODE_GENERATION_FUNC_SPECS.awsCloudFormationLint.id,
    );

    // Add CloudFormationOnly prop to extra
    const extraProp = findPropByName(
      variant.domain,
      "extra",
    ) as ExpandedPropSpecFor["object"];
    if (extraProp) {
      const cfOnlyProp = createScalarProp(
        "CloudFormationOnly",
        "boolean",
        extraProp.metadata.propPath,
        false,
      );
      cfOnlyProp.data.defaultValue = "true";
      cfOnlyProp.data.hidden = true;
      extraProp.entries.push(cfOnlyProp);
    }

    const attrFunc = createAttributeFunc();
    spec.funcs.push(attrFunc);
  }
  return specs;
}

function createAttributeFunc() {
  const code = Deno.readTextFileSync(
    "./src/pipelines/aws/funcs/attribute/awsCloudControlCfAssetAttr.ts",
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
