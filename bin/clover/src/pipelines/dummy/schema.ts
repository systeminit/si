import { CfProperty, SuperSchema } from "../types.ts";

export interface DummySchema extends SuperSchema {
  properties: Record<string, CfProperty>;
  requiredProperties: Set<string>;
}

export const serverSchema: DummySchema = {
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
  handlers: {
    create: { permissions: [], timeoutInMinutes: 60 },
    read: { permissions: [], timeoutInMinutes: 60 },
    update: { permissions: [], timeoutInMinutes: 60 },
    delete: { permissions: [], timeoutInMinutes: 60 },
    list: { permissions: [], timeoutInMinutes: 60 },
  },
};

export const databaseSchema: DummySchema = {
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
  handlers: {
    create: { permissions: [], timeoutInMinutes: 60 },
    read: { permissions: [], timeoutInMinutes: 60 },
    update: { permissions: [], timeoutInMinutes: 60 },
    delete: { permissions: [], timeoutInMinutes: 60 },
    list: { permissions: [], timeoutInMinutes: 60 },
  },
};
