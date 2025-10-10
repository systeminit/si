import type { CfHandler, CfHandlerKind, CfProperty } from "../types.ts";

export type JsonSchema = Record<string, unknown>;

type JSONPointer = string;

export type PropertySet = Set<string>;

export interface AzureOperationData {
  method: string;
  path: string;
  openApiOperation: JsonSchema;
  apiVersion?: string;
}

export type AzureSchema = {
  typeName: string;
  description: string;
  sourceUrl?: string;
  documentationUrl?: string;
  properties: Record<string, CfProperty>;
  requiredProperties: Set<string>;
  primaryIdentifier: JSONPointer[];
  handlers?: Record<CfHandlerKind, CfHandler>;
  apiVersion?: string;
};
