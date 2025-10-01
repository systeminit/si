import type { CfProperty } from "../types.ts";
import { CfHandler, CfHandlerKind } from "../types.ts";

type JSONPointer = string;

export type HetznerSchema = {
  typeName: string;
  description: string;
  sourceUrl?: string;
  documentationUrl?: string;
  properties: Record<string, CfProperty>;
  requiredProperties: Set<string>;
  primaryIdentifier: JSONPointer[];
  handlers?: Record<CfHandlerKind, CfHandler>;
  endpoint?: string;
};

export type JsonSchema = Record<string, unknown>;

export type PropertySet = Set<string>;

export interface OperationData {
  endpoint: string;
  openApiDescription: JsonSchema;
}
