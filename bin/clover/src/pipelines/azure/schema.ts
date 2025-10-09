import type { CfHandler, CfHandlerKind, CfProperty } from "../types.ts";

export type JsonSchema = Record<string, unknown>;

type JSONPointer = string;

export type AzureSchema = {
  typeName: string;
  description: string;
  properties: Record<string, CfProperty>;
  primaryIdentifier: JSONPointer[];
  handlers?: Record<CfHandlerKind, CfHandler>;
};
