import type { CfProperty } from "../types.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";

type JSONPointer = string;

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
  resourcePath?: string;
};

export type JsonSchema = Record<string, unknown>;

export type PropertySet = Set<string>;

export interface AzureOperationData {
  method: string;
  path: string;
  openApiOperation: JsonSchema;
  apiVersion?: string;
}
