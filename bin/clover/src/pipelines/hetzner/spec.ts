import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import _ from "lodash";
import { createDefaultPropFromCf, OnlyProperties } from "../../spec/props.ts";
import type { CfProperty } from "../types.ts";
import {
  CfHandler,
  CfHandlerKind,
  HDB,
  HetznerSchema,
  SuperSchema,
} from "../types.ts";
import { makeModule } from "../generic/index.ts";
import { createDefaultProp } from "./prop.ts";

export type JsonSchema = Record<string, unknown>;
export type PropertySet = Set<string>;

export interface OperationData {
  endpoint: string;
  openApiDescription: JsonSchema;
}

export function pkgSpecFromHetnzer(allSchemas: JsonSchema): ExpandedPkgSpec[] {
  const schemas: HDB = {};
  const specs: ExpandedPkgSpec[] = [];

  // Group all operations by resource noun
  const resourceOperations: Record<string, OperationData[]> = {};
  Object.entries((allSchemas.paths as JsonSchema) || {}).forEach(
    ([endpoint, openApiDescription]) => {
      const noun = endpoint.split("/")[1];
      if (endpoint.includes("actions")) return; // TODO: should we be skipping these?

      if (!resourceOperations[noun]) {
        resourceOperations[noun] = [];
      }
      resourceOperations[noun].push({
        endpoint,
        openApiDescription: openApiDescription as JsonSchema,
      });
    },
  );

  // Process each resource and merge all its operations
  const resourceResults: {
    schema: HetznerSchema;
    onlyProperties: OnlyProperties;
  }[] = [];
  Object.entries(resourceOperations).forEach(([noun, operations]) => {
    const result = mergeResourceOperations(noun, operations);
    if (result) {
      schemas[noun] = result.schema;
      resourceResults.push(result);
    }
  });

  resourceResults.forEach(({ schema, onlyProperties }) => {
    const normalizedOnlyProperties: OnlyProperties = {
      createOnly: normalizeOnlyProperties(onlyProperties.createOnly),
      readOnly: normalizeOnlyProperties(onlyProperties.readOnly),
      writeOnly: normalizeOnlyProperties(onlyProperties.writeOnly),
      primaryIdentifier: onlyProperties.primaryIdentifier,
    };

    const domain = createDefaultProp(
      "domain",
      pruneDomainValues(
        schema.properties as Record<string, CfProperty>,
        onlyProperties,
      ),
      normalizedOnlyProperties,
      schema,
      createDocLink,
    );
    const resourceValue = createDefaultProp(
      "resource_value",
      pruneResourceValues(
        schema.properties as Record<string, CfProperty>,
        onlyProperties,
      ),
      normalizedOnlyProperties,
      schema,
      createDocLink,
    );
    const secrets = createDefaultPropFromCf(
      "secrets",
      {},
      schema,
      onlyProperties,
      createDocLink,
    );

    const module = makeModule(
      schema,
      createDocLink(schema, undefined),
      schema.description,
      domain,
      resourceValue,
      secrets,
      hCategory,
    );
    specs.push(module);
  });

  return specs;
}

export function extractPropertiesFromRequestBody(
  operation: JsonSchema | null,
): { properties: JsonSchema; required: string[] } {
  const schema = (operation?.requestBody as any)?.content?.["application/json"]
    ?.schema as JsonSchema | undefined;

  return {
    properties: (schema?.properties as JsonSchema) || {},
    required: (schema?.required as string[]) || [],
  };
}

export function buildHandlersFromOperations(
  operations: OperationData[],
): {
  handlers: Record<CfHandlerKind, CfHandler>;
  getOperation: JsonSchema | null;
  postOperation: JsonSchema | null;
  putOperation: JsonSchema | null;
  deleteOperation: JsonSchema | null;
} {
  const handlers = {} as Record<CfHandlerKind, CfHandler>;
  let getOperation: JsonSchema | null = null;
  let postOperation: JsonSchema | null = null;
  let putOperation: JsonSchema | null = null;
  let deleteOperation: JsonSchema | null = null;

  operations.forEach(({ openApiDescription }) => {
    const defaultHandler = { permissions: [], timeoutInMinutes: 60 };

    Object.entries(openApiDescription).forEach(([method, operation]) => {
      const op = operation as JsonSchema;
      switch (method) {
        case "get": {
          getOperation = op;
          const opId = op.operationId as string;
          handlers[opId.startsWith("list_") ? "list" : "read"] = defaultHandler;
          break;
        }
        case "put": {
          putOperation = op;
          handlers["update"] = defaultHandler;
          break;
        }
        case "post": {
          postOperation = op;
          handlers["create"] = defaultHandler;
          break;
        }
        case "delete": {
          deleteOperation = op;
          handlers["delete"] = defaultHandler;
          break;
        }
      }
    });
  });

  return {
    handlers,
    getOperation,
    postOperation,
    putOperation,
    deleteOperation,
  };
}

export function normalizeHetznerProperty(prop: JsonSchema): JsonSchema {
  if (prop.type) {
    // normalize nested properties
    const normalized = { ...prop };

    if (normalized.properties) {
      normalized.properties = Object.fromEntries(
        Object.entries(normalized.properties as Record<string, JsonSchema>).map(
          (
            [key, value],
          ) => [key, normalizeHetznerProperty(value)],
        ),
      );
    }

    if (normalized.additionalProperties) {
      normalized.additionalProperties = normalizeHetznerProperty(
        normalized.additionalProperties as JsonSchema,
      );
    }

    if (normalized.items) {
      normalized.items = normalizeHetznerProperty(
        normalized.items as JsonSchema,
      );
    }

    return normalized;
  }

  // handle oneOf with primitive types - smoosh them like cfDb does for array types
  if (prop.oneOf) {
    const allPrimitives = (prop.oneOf as JsonSchema[]).every((member) => {
      const type = member.type;
      return (
        type === "string" ||
        type === "number" ||
        type === "integer" ||
        type === "boolean"
      );
    });

    if (allPrimitives) {
      // Pick the non-string type (prefer number, integer, boolean over string)
      const nonStringMember = (prop.oneOf as JsonSchema[]).find(
        (member) => member.type !== "string",
      );
      const smooshed = nonStringMember
        ? { ...prop, type: nonStringMember.type, oneOf: undefined }
        : { ...prop, type: "string", oneOf: undefined };

      return normalizeHetznerProperty(smooshed);
    }
  }

  return prop;
}

export function mergePropertyDefinitions(
  existing: JsonSchema | undefined,
  newProp: JsonSchema,
): JsonSchema {
  if (!existing) return newProp;

  const merged = { ...existing };

  // Merge enum values if both exist
  const existingEnum = existing.enum as unknown[] | undefined;
  const newPropEnum = newProp.enum as unknown[] | undefined;
  if (existingEnum && newPropEnum) {
    merged.enum = [...new Set([...existingEnum, ...newPropEnum])];
  } else if (newPropEnum) {
    merged.enum = newPropEnum;
  }

  return merged;
}

export function mergeResourceOperations(
  noun: string,
  operations: OperationData[],
): { schema: HetznerSchema; onlyProperties: OnlyProperties } | null {
  // Extract handlers and operations using the dedicated function
  const {
    handlers,
    getOperation,
    postOperation,
    putOperation,
    deleteOperation,
  } = buildHandlersFromOperations(operations);

  // Must have a get operation to proceed
  if (!getOperation) {
    console.error(`No GET operation found for ${noun}`);
    return null;
  }

  // Extract properties from GET response
  const getContent = (getOperation.responses as any)?.["200"]?.content
    ?.["application/json"];
  if (!getContent) {
    console.error(`No JSON response content for GET ${noun}`);
    return null;
  }

  const objShape = Object.values(
    getContent.schema.properties,
  ).pop() as JsonSchema | undefined;
  if (!objShape) {
    console.error("SHAPE EXPECTED", getContent);
    return null;
  }

  const mergedProperties = {
    ...(objShape.properties as JsonSchema),
  };
  const requiredProperties = new Set((objShape.required as string[]) || []);

  // Properties that appear in different operations - for onlyProperties classification
  const getProperties: PropertySet = new Set(
    Object.keys(objShape.properties as JsonSchema),
  );
  const createProperties: PropertySet = new Set();
  const updateProperties: PropertySet = new Set();
  const deleteProperties: PropertySet = new Set();

  // Process the other ops
  [
    { operation: postOperation, propertySet: createProperties, name: "create" },
    { operation: putOperation, propertySet: updateProperties, name: "update" },
    {
      operation: deleteOperation,
      propertySet: deleteProperties,
      name: "delete",
    },
  ].forEach(({ operation, propertySet }) => {
    const { properties: operationProps, required: operationRequired } =
      extractPropertiesFromRequestBody(operation);

    Object.keys(operationProps).forEach((prop) => propertySet.add(prop));

    // Merge properties into the main schema
    Object.entries(operationProps).forEach(([key, prop]) => {
      mergedProperties[key] = mergePropertyDefinitions(
        mergedProperties[key] as JsonSchema,
        prop as JsonSchema,
      );
    });

    // Add required properties
    operationRequired.forEach((prop) => requiredProperties.add(prop));
  });

  // Build onlyProperties
  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier: ["id"],
  };

  // createOnly: only in POST, not in PUT
  createProperties.forEach((prop) => {
    if (
      !updateProperties.has(prop)
    ) {
      onlyProperties.createOnly.push(`/${prop}`);
    }
  });

  // readOnly: only in GET, not in POST/PUT/DELETE
  getProperties.forEach((prop) => {
    if (
      !createProperties.has(prop) && !updateProperties.has(prop) &&
      !deleteProperties.has(prop)
    ) {
      onlyProperties.readOnly.push(`/${prop}`);
    }
  });

  // writeOnly: in POST/PUT/DELETE but not in GET
  const writeProps = [
    ...createProperties,
    ...updateProperties,
    ...deleteProperties,
  ];
  onlyProperties.writeOnly = [
    ...new Set(
      writeProps.filter((prop) => !getProperties.has(prop)).map((prop) =>
        `/${prop}`
      ),
    ),
  ];

  // Normalize properties to handle oneOf with primitives
  const normalizedProperties = Object.fromEntries(
    Object.entries(mergedProperties).map(([key, prop]) => [
      key,
      normalizeHetznerProperty(prop as JsonSchema),
    ]),
  );

  const schema: HetznerSchema = {
    typeName: noun,
    description: "PAUL FIGURE IT OUT",
    properties: normalizedProperties as Record<string, CfProperty>,
    requiredProperties,
    primaryIdentifier: ["id"],
    handlers,
  };

  return { schema, onlyProperties };
}

export function createDocLink(
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  // Hetzner Cloud API reference base URL
  const docLink = "https://docs.hetzner.cloud/reference/cloud";

  // Convert resource name to use dashes (e.g., "ssh_keys" -> "ssh-keys")
  const resourceName = typeName.toLowerCase().replace(/_/g, "-");

  // Build the fragment identifier
  // For definitions (nested types), append the definition name
  if (defName) {
    return `${docLink}#${resourceName}-${defName.toLowerCase()}`;
  }

  // For specific properties, append the property name
  if (propName) {
    return `${docLink}#${resourceName}-${propName.toLowerCase()}`;
  }

  // Base resource link
  return `${docLink}#${resourceName}`;
}

export function hCategory(schema: SuperSchema): string {
  const name = _.camelCase(schema.typeName.replace("_", " "));
  return `Hetzner::${name}`;
}

function normalizeOnlyProperties(props: string[]): string[] {
  const newProps: string[] = [];
  for (const prop of props ?? []) {
    const newProp = prop.split("/").pop();
    if (newProp) {
      newProps.push(newProp);
    }
  }
  return newProps;
}

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
      .filter(
        ([name, prop]) => prop && !readOnlySet.has(`/${name}`),
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
      .filter(
        ([name, prop]) => prop && readOnlySet.has(`/${name}`),
      ),
  );
}
