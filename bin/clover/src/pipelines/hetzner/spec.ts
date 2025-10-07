import _ from "lodash";
import { OnlyProperties } from "../../spec/props.ts";
import type { CfProperty } from "../types.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";
import {
  type HetznerSchema,
  type JsonSchema,
  type OperationData,
  type PropertySet,
} from "./schema.ts";

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

    if (
      normalized.additionalProperties &&
      typeof normalized.additionalProperties === "object"
    ) {
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

  // If there's a type conflict between GET and write operations,
  // prefer the write operation since it will be used for input
  if (existing.type !== newProp.type && newProp.type) {
    return { ...newProp };
  }

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
  allSchemas: JsonSchema,
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

  // Get description from the tag
  const tags = getOperation.tags as string[] | undefined;
  const tagName = tags?.[0];
  let description = `Hetzner Cloud ${noun} resource`;

  if (tagName && allSchemas.tags) {
    const tag = (allSchemas.tags as JsonSchema[]).find(
      (t) => t.name === tagName,
    );
    if (tag?.description) {
      description = tag.description as string;
    }
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

  // Convert noun to PascalCase for the resource name (e.g., "certificates" -> "Certificate")
  const resourceName = _.startCase(_.camelCase(noun)).replace(/ /g, "");

  const schema: HetznerSchema = {
    typeName: `Hetzner::Cloud::${resourceName}`,
    description,
    properties: normalizedProperties as Record<string, CfProperty>,
    requiredProperties,
    primaryIdentifier: ["id"],
    handlers,
    endpoint: noun,
  };

  return { schema, onlyProperties };
}
