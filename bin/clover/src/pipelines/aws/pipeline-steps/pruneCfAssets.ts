import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import _logger from "../../../logger.ts";
import {
  createDefaultFuncSpec,
  createFunc,
  createLeafFuncSpec,
  strippedBase64,
} from "../../../spec/funcs.ts";
import { CF_ONLY_FUNC_SPECS } from "../funcs.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { FuncArgumentSpec } from "../../../bindings/FuncArgumentSpec.ts";
import { AttrFuncInputSpec } from "../../../bindings/AttrFuncInputSpec.ts";
import {
  bfsPropTree,
  createObjectProp,
  createScalarProp,
  ExpandedPropSpec,
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

    // Replace all leaf functions with CF-only codegen and qualification functions
    variant.leafFunctions = [];

    // Add CF-only codegen function
    const codeGenFunc = createDefaultFuncSpec(
      CF_ONLY_FUNC_SPECS.awsCfOnlyCodeGen.displayName,
      CF_ONLY_FUNC_SPECS.awsCfOnlyCodeGen,
      [],
    );
    spec.funcs.push(codeGenFunc);
    variant.leafFunctions.push(
      createLeafFuncSpec(
        "codeGeneration",
        CF_ONLY_FUNC_SPECS.awsCfOnlyCodeGen.id,
        ["domain"],
      ),
    );

    // Add CF-only lint qualification function
    const qualificationFunc = createDefaultFuncSpec(
      CF_ONLY_FUNC_SPECS.awsCfOnlyLint.displayName,
      CF_ONLY_FUNC_SPECS.awsCfOnlyLint,
      [],
    );
    spec.funcs.push(qualificationFunc);
    variant.leafFunctions.push(
      createLeafFuncSpec(
        "qualification",
        CF_ONLY_FUNC_SPECS.awsCfOnlyLint.id,
        ["domain"],
      ),
    );

    // Restructure domain: wrap all CF props under "properties", keep "extra" as sibling
    restructureDomainProps(variant.domain, spec.name);

    // Add CloudFormationOnly prop to extra (extra is always created by restructureDomainProps)
    const extraProp = findPropByName(
      variant.domain,
      "extra",
    ) as ExpandedPropSpecFor["object"];
    const cfOnlyProp = createScalarProp(
      "CloudFormationOnly",
      "boolean",
      extraProp.metadata.propPath,
      false,
    );
    cfOnlyProp.data.defaultValue = "true";
    cfOnlyProp.data.hidden = true;
    extraProp.entries.push(cfOnlyProp);

    const attrFunc = createAttributeFunc();
    spec.funcs.push(attrFunc);
  }
  return specs;
}

function createAttributeFunc() {
  const code = Deno.readTextFileSync(
    "./src/pipelines/aws/funcs/attribute/awsCfOnlyAttr.ts",
  );
  const codeBase64: string = strippedBase64(code);
  const args: FuncArgumentSpec[] = [
    {
      name: "properties",
      kind: "object",
      elementKind: null,
      uniqueId: ulid(),
      deleted: false,
    },
    {
      name: "extra",
      kind: "object",
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
    "a2b5a6a39ce093738e0e19a55ecfaeca5bd99716519163c8c15f3674d95d096e",
    args,
  );
}

/**
 * Restructures the domain prop for CF-only assets:
 * - Creates a new "properties" object under domain
 * - Moves all CF props (everything except "extra") into "properties"
 * - Adds "AwsResourceType" to "extra" with the schema name
 */
function restructureDomainProps(
  domain: ExpandedPropSpecFor["object"],
  schemaName: string,
) {
  // Separate extra from the other props, or create it if it doesn't exist
  let extraProp = domain.entries.find(
    (p) => p.name === "extra",
  ) as ExpandedPropSpecFor["object"] | undefined;
  const cfProps = domain.entries.filter((p) => p.name !== "extra");

  // Create extra prop if it doesn't exist
  if (!extraProp) {
    extraProp = createObjectProp("extra", ["root", "domain"], undefined, false);
  }

  // Create the new "properties" object prop
  const propertiesProp = createObjectProp(
    "properties",
    ["root", "domain"],
    undefined,
    false,
  );

  // Move CF props into properties and update their paths
  for (const prop of cfProps) {
    updatePropPaths(prop, ["root", "domain"], ["root", "domain", "properties"]);
    propertiesProp.entries.push(prop);
  }

  // Add or update AwsResourceType in extra
  {
    const existingAwsResourceType = extraProp.entries.find(
      (p) => p.name === "AwsResourceType",
    );
    if (existingAwsResourceType) {
      // Update existing prop's defaultValue
      existingAwsResourceType.data.defaultValue = schemaName;
      existingAwsResourceType.data.hidden = true;
    } else {
      // Create new prop if it doesn't exist
      const awsResourceTypeProp = createScalarProp(
        "AwsResourceType",
        "string",
        extraProp.metadata.propPath,
        false,
      );
      awsResourceTypeProp.data.defaultValue = schemaName;
      awsResourceTypeProp.data.hidden = true;
      extraProp.entries.push(awsResourceTypeProp);
    }
  }

  // Create CloudFormationResourceBody prop (output for attribute function)
  const cloudFormationResourceBodyProp = createJsonProp(
    "CloudFormationResourceBody",
    ["root", "domain"],
    false,
  );

  // Bind the attribute function to the output prop
  cloudFormationResourceBodyProp.data.funcUniqueId =
    "a2b5a6a39ce093738e0e19a55ecfaeca5bd99716519163c8c15f3674d95d096e";
  cloudFormationResourceBodyProp.data.inputs = [
    {
      kind: "prop",
      name: "properties",
      prop_path: "/root/domain/properties",
      unique_id: null,
      deleted: false,
    },
    {
      kind: "prop",
      name: "extra",
      prop_path: "/root/domain/extra",
      unique_id: null,
      deleted: false,
    },
  ] as AttrFuncInputSpec[];

  // Rebuild domain entries: properties, extra, CloudFormationResourceBody
  domain.entries = [propertiesProp, extraProp, cloudFormationResourceBodyProp];
}

/**
 * Updates the propPath for a prop and all its descendants
 */
function updatePropPaths(
  prop: ExpandedPropSpec,
  oldPrefix: string[],
  newPrefix: string[],
) {
  bfsPropTree(prop, (p) => {
    // Replace the old prefix with the new prefix in the propPath
    const oldPath = p.metadata.propPath;
    if (
      oldPath.length >= oldPrefix.length &&
      oldPath.slice(0, oldPrefix.length).join("/") === oldPrefix.join("/")
    ) {
      p.metadata.propPath = [...newPrefix, ...oldPath.slice(oldPrefix.length)];
    }
  });
}

/**
 * Creates a JSON prop with CodeEditor widget
 */
function createJsonProp(
  name: string,
  parentPath: string[],
  required: boolean,
): ExpandedPropSpec {
  const data: ExpandedPropSpec["data"] = {
    name,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: [],
    widgetKind: "CodeEditor",
    widgetOptions: null,
    hidden: false,
    docLink: null,
    documentation: null,
    uiOptionals: null,
  };

  const prop: ExpandedPropSpec = {
    kind: "json",
    data,
    name,
    uniqueId: ulid(),
    metadata: {
      createOnly: false,
      readOnly: false,
      writeOnly: false,
      primaryIdentifier: false,
      propPath: [...parentPath, name],
      required,
    },
    cfProp: undefined,
  };

  return prop;
}
