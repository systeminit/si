import { SuperSchema } from "../types.ts";

export interface GcpDiscoveryDocument {
  kind: "discovery#restDescription";
  name: string;
  version: string;
  title: string;
  description?: string;
  documentationLink?: string;
  baseUrl: string;
  basePath: string;
  rootUrl: string;
  servicePath: string;
  parameters?: Record<string, GcpParameter>;
  auth?: GcpAuth;
  schemas: Record<string, GcpSchemaDefinition>;
  resources?: Record<string, GcpResource>;
  methods?: Record<string, GcpMethod>;
}

export interface GcpResource {
  methods?: Record<string, GcpMethod>;
  resources?: Record<string, GcpResource>;
}

export interface GcpMethod {
  id: string;
  path: string;
  httpMethod: string;
  description?: string;
  parameters?: Record<string, GcpParameter>;
  parameterOrder?: string[];
  request?: { $ref: string };
  response?: { $ref: string };
  scopes?: string[];
  supportsMediaDownload?: boolean;
  supportsMediaUpload?: boolean;
}

export interface GcpParameter {
  type: string;
  description?: string;
  required?: boolean;
  location: "path" | "query";
  pattern?: string;
  minimum?: string;
  maximum?: string;
  default?: string;
}

export interface GcpAuth {
  oauth2?: {
    scopes: Record<string, { description: string }>;
  };
}

export interface GcpSchemaDefinition {
  id?: string;
  type?: string;
  description?: string;
  properties?: Record<string, any>;
  required?: string[];
  additionalProperties?: any;
  items?: any;
  $ref?: string;
}

export interface GcpSchema extends SuperSchema {
  requiredProperties: Set<string>;
  service: string;
  version: string;
  resourcePath: string[];
  baseUrl: string;
  documentationLink?: string;
}
