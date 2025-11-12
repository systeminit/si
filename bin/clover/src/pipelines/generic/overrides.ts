import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  createScalarProp,
  ExpandedPropSpec,
  ExpandedPropSpecFor,
  findPropByName,
  PropPath,
  propPathStr,
  toPropPathArray,
} from "../../spec/props.ts";
import { PropUsageMap } from "../aws/pipeline-steps/addDefaultPropsAndSockets.ts";
import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";
import { ActionFuncSpec } from "../../bindings/ActionFuncSpec.ts";
import { LeafFunctionSpec } from "../../bindings/LeafFunctionSpec.ts";
import {
  createActionFuncSpec,
  createFunc,
  createLeafFuncSpec,
  createManagementFuncSpec,
  strippedBase64,
} from "../../spec/funcs.ts";
import { PropSpecWidgetKind } from "../../bindings/PropSpecWidgetKind.ts";
import { PropOverrideFn, SchemaOverrideFn } from "../types.ts";
import { ManagementFuncSpec } from "../../bindings/ManagementFuncSpec.ts";

/**
 * Shared utility functions for asset overrides across all providers
 */

export function propForOverride(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpec {
  const prop = findPropByName(objPropSpec, propName);
  if (!prop) {
    throw new Error(
      `Prop ${propName} not found under ${objPropSpec.name} for override!`,
    );
  }
  return prop;
}

export function arrayPropForOverride(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpecFor["array"] {
  const prop = propForOverride(objPropSpec, propName);
  if (prop?.kind !== "array") {
    throw new Error(`Prop ${propName} is not an array!`);
  }
  return prop;
}

export function objectPropForOverride(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpecFor["object"] {
  const prop = propForOverride(objPropSpec, propName);
  if (prop?.kind !== "object") {
    throw new Error(`Prop ${propName} is not an object!`);
  }
  return prop;
}

export function stringPropForOverride(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpecFor["string"] {
  const prop = propForOverride(objPropSpec, propName);
  if (prop?.kind !== "string") {
    throw new Error(`Prop ${propName} is not a string!`);
  }
  return prop;
}

export function attachExtraManagementFunction(
  funcPath: string,
  name: string,
  uniqueId: string,
): { func: FuncSpec; mgmtFuncSpec: ManagementFuncSpec } {
  const funcCode = Deno.readTextFileSync(funcPath);
  const func = createFunc(
    name,
    "management",
    "management",
    strippedBase64(funcCode),
    uniqueId,
    [],
  );
  func.data!.displayName = name;

  const mgmtFuncSpec = createManagementFuncSpec(name, func.uniqueId);

  return { func, mgmtFuncSpec };
}

export function attachExtraActionFunction(
  funcPath: string,
  name: string,
  kind: ActionFuncSpecKind,
  uniqueId: string,
): { func: FuncSpec; actionFuncSpec: ActionFuncSpec } {
  const funcCode = Deno.readTextFileSync(funcPath);
  const func = createFunc(
    name,
    "jsAction",
    "action",
    strippedBase64(funcCode),
    uniqueId,
    [],
  );
  func.data!.displayName = name;

  const actionFuncSpec = createActionFuncSpec(kind, func.uniqueId);

  return { func, actionFuncSpec };
}

export function attachQualificationFunction(
  funcPath: string,
  name: string,
  uniqueId: string,
  domainId: string,
): { func: FuncSpec; leafFuncSpec: LeafFunctionSpec } {
  const funcCode = Deno.readTextFileSync(funcPath);

  const func = createFunc(
    name,
    "jsAttribute",
    "qualification",
    strippedBase64(funcCode),
    uniqueId,
    [
      {
        name: "domain",
        kind: "object",
        elementKind: null,
        uniqueId: domainId,
        deleted: false,
      },
    ],
  );
  func.data!.displayName = name;

  const leafFuncSpec = createLeafFuncSpec("qualification", func.uniqueId, [
    "domain",
  ]);

  return { func, leafFuncSpec };
}

export function addSecretProp(
  secretKind: string,
  secretKey: string,
  propPath: string[],
) {
  return (spec: ExpandedPkgSpec) => {
    const variant = spec.schemas[0].variants[0];

    const [secretName] = propPath.slice(-1);
    if (!secretName) {
      return;
    }

    // Find secret prop
    let secretParent = variant.domain;
    let secretProp: ExpandedPropSpec | undefined = variant.domain;

    for (const propName of propPath) {
      // If we haven't found the secret prop yet, and we're not with an object in hand, break
      if (secretProp.kind !== "object") {
        secretProp = undefined;
        break;
      }

      secretParent = secretProp;
      const thisProp = secretParent.entries.find((p) => p.name === propName);

      // If we don't find the prop on the parent, break
      if (!thisProp) {
        secretProp = undefined;
        break;
      }

      secretProp = thisProp;
    }

    if (!secretProp) {
      console.log(`Could not add secret value for ${spec.name}`);
      return;
    }

    // Find propUsageMap
    const extraProp = variant.domain.entries.find((p) => p.name === "extra");
    if (extraProp?.kind !== "object") {
      return;
    }
    const propUsageMapProp = extraProp.entries.find(
      (p) => p.name === "PropUsageMap",
    );
    const defaultValue = propUsageMapProp?.data.defaultValue;
    const propUsageMap = JSON.parse(
      typeof defaultValue === "string" ? defaultValue : "{}",
    ) as PropUsageMap;

    if (!propUsageMapProp || !Array.isArray(propUsageMap?.secrets)) {
      return;
    }

    // Remove secret from the domain tree
    secretParent.entries = secretParent.entries.filter(
      (p: ExpandedPropSpec) => p.name !== secretName,
    );

    // Add prop to secrets tree
    secretProp.data.widgetKind = "Secret";
    secretProp.data.widgetOptions = [
      {
        label: "secretKind",
        value: secretKind,
      },
    ];
    variant.secrets.entries.push(secretProp);
    // Replace "domain" with "secrets" on propPath
    secretProp.metadata.propPath[1] = "secrets";

    // add secret to the propUsageMap
    propUsageMap.secrets.push({
      secretKey,
      propPath,
    });
    propUsageMapProp.data.defaultValue = JSON.stringify(propUsageMap);
  };
}

// Generic policy document override
export function policyDocumentProp(prop: ExpandedPropSpec) {
  if (prop.kind !== "string" && prop.kind !== "json") {
    throw new Error(`${prop.metadata.propPath} is not a string`);
  }
  prop.kind = "json";
  prop.data.widgetKind = "CodeEditor";
  addPropSuggestSource(prop, {
    schema: "String Template",
    prop: "/domain/Rendered/Value",
  });
}

// Generic ARN prop override
export function arnProp(suggestSchema: string, suggestProp: string = "Arn") {
  return suggest(suggestSchema, suggestProp);
}

// Generic suggestion override. If prop does not start with /, it is assumed to be under /resource_value
export function suggest(suggestSchema: string, suggestProp: string) {
  if (!suggestProp.startsWith("/")) {
    suggestProp = `/resource_value/${suggestProp}`;
  }
  return (addToProp: ExpandedPropSpec, spec: ExpandedPkgSpec) => {
    // Don't add self-suggestions
    if (
      !(spec.name === suggestSchema && propPathStr(addToProp) === suggestProp)
    ) {
      addPropSuggestSource(addToProp, {
        schema: suggestSchema,
        prop: suggestProp,
      });
    }
  };
}

// Generate a function to fix schema or category names for a schema
export function fixNames(
  names: { schemaName?: string; categoryName?: string },
) {
  return (spec: ExpandedPkgSpec) => {
    const schema = spec.schemas[0];
    if (!schema.data) {
      throw new Error(
        "Schema data should exist for Microsoft.Aad/domainServices/ouContainer",
      );
    }
    if (names.categoryName) {
      schema.data.category = names.categoryName;
    }
    if (names.schemaName) {
      spec.name = names.schemaName;
      schema.name = names.schemaName;
      schema.data.name = names.schemaName;
      for (const v of schema.variants) {
        v.data.displayName = names.schemaName;
      }
    }
  };
}

// ComboBox override with fixed set of options. Options can be specified any of these ways:
//
// - widget("TextArea")
// - widget("ComboBox", ["a", "b", ...])
// - comboBox({ "a": "label A", ... })
// - comboBox([ { label: "label A", value: "a" }, ... ])
//
export function widget(
  kind: Exclude<PropSpecWidgetKind, "ComboBox">,
): PropOverrideFn;
export function widget(
  kind: "ComboBox",
  options?:
    | (string | { label: string; value: string })[]
    | Record<string, string>,
): PropOverrideFn;
export function widget(
  kind: PropSpecWidgetKind,
  options?:
    | (string | { label: string; value: string })[]
    | Record<string, string>,
) {
  if (options && !Array.isArray(options)) {
    options = Object.entries(options).map(([value, label]) => ({
      label,
      value,
    }));
  }
  return (prop: ExpandedPropSpec) => {
    prop.data.widgetKind = kind;
    if (options) {
      prop.data.widgetOptions = options.map((o) =>
        typeof o === "string" ? { label: o, value: o } : o
      );
    }
  };
}

export function addQualificationFunction(
  funcPath: string,
  name: string,
  uniqueId: string,
): SchemaOverrideFn {
  return (spec) => {
    const variant = spec.schemas[0].variants[0];
    if (!variant.domain.uniqueId) {
      throw new Error("Expected domain.uniqueId");
    }
    const { func, leafFuncSpec } = attachQualificationFunction(
      funcPath,
      name,
      uniqueId,
      variant.domain.uniqueId,
    );

    spec.funcs.push(func);
    variant.leafFunctions.push(leafFuncSpec);
  };
}
export function addScalarProp(
  propPath: PropPath,
  kind: "string" | "number" | "boolean",
  required = false,
): SchemaOverrideFn {
  return (spec: ExpandedPkgSpec) => {
    const parentPathArray = toPropPathArray(propPath);
    const propName = parentPathArray.pop();
    if (!propName) {
      throw new Error(`propPath too short: ${propPath}`);
    }
    const parentProp = findObjectProp(spec, parentPathArray);
    parentProp.entries.push(
      createScalarProp(propName, kind, parentPathArray, required),
    );
  };
}

// Drill down object props based on the given propPath
function findObjectProp(
  spec: ExpandedPkgSpec,
  propPathArray: ExpandedPropSpec["metadata"]["propPath"],
): ExpandedPropSpecFor["object"] {
  const variant = spec.schemas[0].variants[0];
  // Split into first part, middle parts, last part
  const [root, start, ...pathParts] = propPathArray;
  if (root !== "root") {
    throw new Error(`propPath must start with /root/: propPathArray`);
  }

  // Get the first prop (domain or resource_value) so we can drill down from there
  let objectProp;
  switch (start) {
    case "domain":
      objectProp = variant.domain;
      break;
    case "resource_value":
      objectProp = variant.resourceValue;
      break;
    default:
      throw new Error(`Invalid start of propPath: ${start}`);
  }

  // Drill down to the parent of the target prop
  for (const part of pathParts) {
    objectProp = findPropByName(objectProp, part);
    if (objectProp?.kind !== "object") {
      throw new Error(
        `Could not find object prop ${part} under ${propPathArray}`,
      );
    }
  }

  return objectProp;
}
