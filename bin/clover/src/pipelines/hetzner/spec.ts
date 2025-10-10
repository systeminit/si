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
} {
  const handlers = {} as Record<CfHandlerKind, CfHandler>;
  let getOperation: JsonSchema | null = null;
  let postOperation: JsonSchema | null = null;
  let putOperation: JsonSchema | null = null;

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
): {
  schema: HetznerSchema;
  onlyProperties: OnlyProperties;
  domainProperties: Record<string, CfProperty>;
  resourceValueProperties: Record<string, CfProperty>;
} | null {
  const {
    handlers,
    getOperation,
    postOperation,
    putOperation,
  } = buildHandlersFromOperations(operations);

  // Get description from the tag
  const tags = getOperation?.tags as string[] | undefined;
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

  // Extract properties from CREATE (POST) request
  if (!postOperation) return null;
  const createContent = (postOperation.requestBody as any)?.content
    ?.["application/json"];
  if (!createContent) {
    console.error(`No JSON response content for Create ${noun}`);
    return null;
  }

  // Start with POST properties for domain (writable properties)
  const domainProperties = {
    ...(createContent.schema?.properties as JsonSchema),
  };
  const requiredProperties = new Set(
    (createContent.schema?.required as string[]) || [],
  );

  // Properties that appear in different operations - for onlyProperties classification
  const createProperties: PropertySet = new Set(
    Object.keys(createContent.schema?.properties as JsonSchema),
  );

  const getContent = (getOperation?.responses as any)?.["200"]?.content
    ?.["application/json"];
  const getObjShape = getContent?.schema?.properties
    ? Object.values(getContent.schema.properties).pop() as
      | JsonSchema
      | undefined
    : undefined;
  const getProperties: PropertySet = new Set(
    Object.keys((getObjShape?.properties as JsonSchema) || {}),
  );

  const updateProperties: PropertySet = new Set();

  // Merge PUT into domain (POST + PUT = writable properties)
  {
    const { properties: operationProps, required: operationRequired } =
      extractPropertiesFromRequestBody(putOperation);
    Object.keys(operationProps).forEach((prop) => updateProperties.add(prop));
    Object.entries(operationProps).forEach(([key, prop]) => {
      domainProperties[key] = mergePropertyDefinitions(
        domainProperties[key] as JsonSchema,
        prop as JsonSchema,
      );
    });
    // Add required properties from UPDATE operation
    operationRequired.forEach((prop) => requiredProperties.add(prop));
  }

  const resourceValueProperties = {
    ...(getObjShape?.properties as JsonSchema || {}),
  };

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
      onlyProperties.createOnly.push(prop);
    }
  });

  // readOnly: in GET but not in POST or PUT
  getProperties.forEach((prop) => {
    if (!createProperties.has(prop) && !updateProperties.has(prop)) {
      onlyProperties.readOnly.push(prop);
    }
  });

  // writeOnly: in POST/PUT but not in GET
  const writeProps = [
    ...createProperties,
    ...updateProperties,
  ];
  onlyProperties.writeOnly = [
    ...new Set(
      writeProps.filter((prop) => !getProperties.has(prop)),
    ),
  ];

  // Normalize domain properties (POST + PUT = writable)
  const normalizedDomainProperties = Object.fromEntries(
    Object.entries(domainProperties).map(([key, prop]) => [
      key,
      normalizeHetznerProperty(prop as JsonSchema),
    ]),
  );

  // Normalize resource_value properties (GET response = readable)
  const normalizedResourceValueProperties = Object.fromEntries(
    Object.entries(resourceValueProperties).map(([key, prop]) => [
      key,
      normalizeHetznerProperty(prop as JsonSchema),
    ]),
  );

  const mergedProperties = { ...normalizedDomainProperties };

  // Convert noun to PascalCase for the resource name (e.g., "certificates" -> "Certificate")
  const resourceName = _.startCase(_.camelCase(noun)).replace(/ /g, "");

  const schema: HetznerSchema = {
    typeName: `Hetzner::Cloud::${resourceName}`,
    description,
    properties: mergedProperties as Record<string, CfProperty>,
    requiredProperties,
    primaryIdentifier: ["id"],
    handlers,
    endpoint: noun,
  };

  return {
    schema,
    onlyProperties,
    domainProperties: normalizedDomainProperties as Record<string, CfProperty>,
    resourceValueProperties: normalizedResourceValueProperties as Record<
      string,
      CfProperty
    >,
  };
}
