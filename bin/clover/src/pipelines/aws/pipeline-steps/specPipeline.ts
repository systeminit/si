import { CfProperty, CfSchema } from "../../../cfDb.ts";
import {
  createDefaultPropFromCf,
  createDocLink,
  OnlyProperties,
} from "../../../spec/props.ts";
import {
  ExpandedPkgSpec,
} from "../../../spec/pkgs.ts";
import { makeModule } from "../../generic/index.ts";

export function cfCategory(schema: CfSchema): string {
  const [metaCategory, category] = schema.typeName.split("::");
  return `${metaCategory}::${category}`;
}

export function pkgSpecFromCf(cfSchema: CfSchema): ExpandedPkgSpec {
  const [metaCategory, category, name] = cfSchema.typeName.split("::");

  if (!["AWS", "Alexa"].includes(metaCategory) || !category || !name) {
    throw `Bad typeName: ${cfSchema.typeName}`;
  }

  const onlyProperties: OnlyProperties = {
    createOnly: normalizeOnlyProperties(cfSchema.createOnlyProperties),
    readOnly: normalizeOnlyProperties(cfSchema.readOnlyProperties),
    writeOnly: normalizeOnlyProperties(cfSchema.writeOnlyProperties),
    primaryIdentifier: normalizeOnlyProperties(cfSchema.primaryIdentifier),
  };

  const domain = createDefaultPropFromCf(
    "domain",
    pruneDomainValues(cfSchema.properties, onlyProperties),
    cfSchema,
    onlyProperties,
  );

  const resourceValue = createDefaultPropFromCf(
    "resource_value",
    pruneResourceValues(cfSchema.properties, onlyProperties),
    cfSchema,
    onlyProperties,
  );

  const secrets =  createDefaultPropFromCf("secrets", {}, cfSchema, onlyProperties);

  return makeModule(
    cfSchema,
    createDocLink(cfSchema, undefined),
    cfSchema.description,
    domain,
    resourceValue,
    secrets,
    cfCategory,
  )
}
// Remove all read only props from this list, since readonly props go on the
// resource value tree
function pruneDomainValues(
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
): Record<string, CfProperty> {
  if (!properties || !onlyProperties.readOnly) {
    return {};
  }

  const readOnlySet = new Set(onlyProperties.readOnly);
  return Object.fromEntries(
    Object.entries(properties)
      // Include properties that either have a type OR have oneOf/anyOf
      .filter(
        ([name, prop]) =>
          (prop.type || prop.oneOf || prop.anyOf) && !readOnlySet.has(name),
      ),
  );
}

function pruneResourceValues(
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
): Record<string, CfProperty> {
  if (!properties || !onlyProperties?.readOnly) {
    return {};
  }

  const readOnlySet = new Set(onlyProperties.readOnly);
  return Object.fromEntries(
    Object.entries(properties)
      // Include properties that either have a type OR have oneOf/anyOf
      .filter(
        ([name, prop]) =>
          (prop.type || prop.oneOf || prop.anyOf) && readOnlySet.has(name),
      ),
  );
}

function normalizeOnlyProperties(props: string[] | undefined): string[] {
  const newProps: string[] = [];
  for (const prop of props ?? []) {
    const newProp = prop.split("/").pop();
    if (newProp) {
      newProps.push(newProp);
    }
  }
  return newProps;
}
