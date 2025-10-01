import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { createDefaultPropFromCf, OnlyProperties } from "../../spec/props.ts";
import type { CfProperty } from "../types.ts";
import { SuperSchema } from "../types.ts";
import { makeModule } from "../generic/index.ts";

function createDocLink(
  _schema: SuperSchema,
  _defName: string | undefined,
  _propName?: string,
): string {
  return "https://dummy.example.com/docs";
}

// Mock schema for a simple Server resource
const serverSchema: SuperSchema = {
  typeName: "Dummy::Server",
  description: "A dummy server resource for testing",
  properties: {
    id: {
      type: "string",
      description: "Unique identifier for the server",
    },
    name: {
      type: "string",
      description: "Name of the server",
    },
    size: {
      type: "string",
      description: "Size of the server (small, medium, large)",
      enum: ["small", "medium", "large"],
    },
    region: {
      type: "string",
      description: "Region where the server is located",
    },
    status: {
      type: "string",
      description: "Current status of the server",
      enum: ["running", "stopped", "pending"],
    },
    ipAddress: {
      type: "string",
      description: "IP address of the server",
    },
  },
  requiredProperties: new Set(["name", "size", "region"]),
  primaryIdentifier: ["id"],
  handlers: {
    create: { permissions: [], timeoutInMinutes: 60 },
    read: { permissions: [], timeoutInMinutes: 60 },
    update: { permissions: [], timeoutInMinutes: 60 },
    delete: { permissions: [], timeoutInMinutes: 60 },
    list: { permissions: [], timeoutInMinutes: 60 },
  },
};

// Mock schema for a simple Database resource
const databaseSchema: SuperSchema = {
  typeName: "Dummy::Database",
  description: "A dummy database resource for testing",
  properties: {
    id: {
      type: "string",
      description: "Unique identifier for the database",
    },
    name: {
      type: "string",
      description: "Name of the database",
    },
    engine: {
      type: "string",
      description: "Database engine (postgres, mysql, mongodb)",
      enum: ["postgres", "mysql", "mongodb"],
    },
    version: {
      type: "string",
      description: "Database engine version",
    },
    size: {
      type: "integer",
      description: "Storage size in GB",
    },
    port: {
      type: "integer",
      description: "Database port number",
    },
    status: {
      type: "string",
      description: "Current status of the database",
      enum: ["available", "creating", "deleting"],
    },
  },
  requiredProperties: new Set(["name", "engine"]),
  primaryIdentifier: ["id"],
  handlers: {
    create: { permissions: [], timeoutInMinutes: 60 },
    read: { permissions: [], timeoutInMinutes: 60 },
    update: { permissions: [], timeoutInMinutes: 60 },
    delete: { permissions: [], timeoutInMinutes: 60 },
    list: { permissions: [], timeoutInMinutes: 60 },
  },
};

export function pkgSpecFromDummy(): ExpandedPkgSpec[] {
  const schemas = [serverSchema, databaseSchema];
  const specs: ExpandedPkgSpec[] = [];

  for (const schema of schemas) {
    // Define onlyProperties for each resource
    const onlyProperties: OnlyProperties = {
      createOnly: [],
      readOnly: ["id", "ipAddress", "status"],
      writeOnly: [],
      primaryIdentifier: ["id"],
    };

    // Separate domain properties (writable) from resource values (read-only)
    const domainProperties: Record<string, CfProperty> = {};
    const resourceValueProperties: Record<string, CfProperty> = {};

    for (const [key, prop] of Object.entries(schema.properties)) {
      if (onlyProperties.readOnly.includes(key)) {
        resourceValueProperties[key] = prop as CfProperty;
      } else {
        domainProperties[key] = prop as CfProperty;
      }
    }

    const domain = createDefaultPropFromCf(
      "domain",
      domainProperties,
      schema,
      onlyProperties,
      createDocLink,
    );

    const resourceValue = createDefaultPropFromCf(
      "resource_value",
      resourceValueProperties,
      schema,
      onlyProperties,
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
      "https://example.com/dummy/docs",
      schema.description,
      domain,
      resourceValue,
      secrets,
      dummyCategory,
    );

    specs.push(module);
  }

  return specs;
}

function dummyCategory(schema: SuperSchema): string {
  return schema.typeName;
}