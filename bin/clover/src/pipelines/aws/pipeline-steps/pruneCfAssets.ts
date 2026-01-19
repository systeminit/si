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
import { propPathToString } from "../../../spec/sockets.ts";

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

    // Add CloudFormationResourceBody prop wired to the attribute function
    if (extraProp) {
      const resourceBodyProp = createScalarProp(
        "CloudFormationResourceBody",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      resourceBodyProp.data.hidden = true;
      resourceBodyProp.data.widgetKind = "CodeEditor";
      resourceBodyProp.data.funcUniqueId = attrFunc.uniqueId;
      resourceBodyProp.data.inputs = [
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
      ];
      extraProp.entries.push(resourceBodyProp);
    }
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
  ];

  return createFunc(
    "si:cfResourceBodyBuilder",
    "jsAttribute",
    "json",
    codeBase64,
    "e5a9bd4e01cc98f6a1b568210b8a421d842507a05572a913bf8a3fa34c8b5862",
    args,
  );
}
