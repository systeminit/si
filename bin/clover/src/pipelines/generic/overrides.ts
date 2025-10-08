import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import {
  addPropSuggestSource,
  ExpandedPropSpec,
  ExpandedPropSpecFor,
  findPropByName,
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
  strippedBase64,
} from "../../spec/funcs.ts";

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
  return (addToProp: ExpandedPropSpec) =>
    addPropSuggestSource(addToProp, {
      schema: suggestSchema,
      prop: suggestProp,
    });
}
