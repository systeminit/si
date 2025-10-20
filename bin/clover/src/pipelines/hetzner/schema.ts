import type { SuperSchema } from "../types.ts";

export interface HetznerSchema extends SuperSchema {
  requiredProperties: Set<string>;
  endpoint?: string;
}

export type JsonSchema = Record<string, unknown>;

export type PropertySet = Set<string>;

export interface OperationData {
  endpoint: string;
  openApiDescription: JsonSchema;
}
