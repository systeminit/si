import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { OnlyProperties } from "../../spec/props.ts";
import { makeModule, normalizeOnlyProperties } from "../generic/index.ts";
import { CfProperty } from "../types.ts";
import { awsProviderConfig } from "./provider.ts";
import type { CfDb, CfSchema } from "./schema.ts";

function pruneProperties(
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
  keepReadOnly: boolean,
  pathPrefix: string = "",
): Record<string, CfProperty> {
  const readOnlySet = new Set(onlyProperties.readOnly);
  const result: Record<string, CfProperty> = {};

  for (const [name, prop] of Object.entries(properties)) {
    if (!prop) continue;

    const cfProp = prop as CfProperty;
    const currentPath = pathPrefix ? `${pathPrefix}/${name}` : name;
    // Check both the simple name and the full path to handle both normalized and non-normalized readOnly lists
    const isReadOnly = readOnlySet.has(name) || readOnlySet.has(currentPath);
    const shouldKeep = keepReadOnly ? isReadOnly : !isReadOnly;

    // Check if this is an object with nested properties
    if (
      typeof cfProp === "object" && cfProp !== null && "properties" in cfProp &&
      cfProp.properties
    ) {
      const prunedChildren = pruneProperties(
        cfProp.properties as Record<string, CfProperty>,
        onlyProperties,
        keepReadOnly,
        currentPath,
      );

      // Only include if this object has matching children
      // Never include empty objects - they are pruned entirely
      if (Object.keys(prunedChildren).length > 0) {
        result[name] = { ...cfProp, properties: prunedChildren };
      }
    } else if (shouldKeep) {
      result[name] = cfProp;
    }
  }

  return result;
}

function cleanProperties(
  properties: Record<string, CfProperty>,
): Record<string, CfProperty> {
  const cleaned: Record<string, CfProperty> = {};
  for (const [name, prop] of Object.entries(properties)) {
    const cfProp = prop as CfProperty;
    if (cfProp.type || cfProp.oneOf || cfProp.anyOf) {
      cleaned[name] = cfProp;
    }
  }
  return cleaned;
}

function splitAwsProperties(
  schema: CfSchema,
  onlyProperties: OnlyProperties,
): {
  domainProperties: Record<string, CfProperty>;
  resourceValueProperties: Record<string, CfProperty>;
} {
  const domainProperties = cleanProperties(
    pruneProperties(
      schema.properties as Record<string, CfProperty>,
      onlyProperties,
      false, // keep writable properties
    ),
  );

  const resourceValueProperties = cleanProperties(
    pruneProperties(
      schema.properties as Record<string, CfProperty>,
      onlyProperties,
      true, // keep readOnly properties
    ),
  );

  return { domainProperties, resourceValueProperties };
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

  const { domainProperties, resourceValueProperties } = splitAwsProperties(
    cfSchema,
    onlyProperties,
  );

  return makeModule(
    cfSchema,
    cfSchema.description,
    onlyProperties,
    awsProviderConfig,
    domainProperties,
    resourceValueProperties,
  );
}

export function parseSchema(rawSchema: unknown): ExpandedPkgSpec[] {
  const cfDb = rawSchema as CfDb;
  const specs: ExpandedPkgSpec[] = [];

  for (const cfSchema of Object.values(cfDb)) {
    const [metaCategory, category, name] = cfSchema.typeName.split("::");

    if (!["AWS", "Alexa"].includes(metaCategory) || !category || !name) {
      console.log(`Skipping invalid typeName: ${cfSchema.typeName}`);
      continue;
    }

    const onlyProperties: OnlyProperties = {
      createOnly: normalizeOnlyProperties(cfSchema.createOnlyProperties),
      readOnly: normalizeOnlyProperties(cfSchema.readOnlyProperties),
      writeOnly: normalizeOnlyProperties(cfSchema.writeOnlyProperties),
      primaryIdentifier: normalizeOnlyProperties(cfSchema.primaryIdentifier),
    };

    const { domainProperties, resourceValueProperties } = splitAwsProperties(
      cfSchema,
      onlyProperties,
    );

    try {
      const spec = makeModule(
        cfSchema,
        cfSchema.description,
        onlyProperties,
        awsProviderConfig,
        domainProperties,
        resourceValueProperties,
      );

      specs.push(spec);
    } catch (error) {
      console.log(`Error processing ${cfSchema.typeName}: ${error}`);
      continue;
    }
  }

  return specs;
}
