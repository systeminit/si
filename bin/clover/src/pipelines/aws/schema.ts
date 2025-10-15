import type { JSONSchema } from "../draft_07.ts";
import type { CfProperty, CfHandler, CfHandlerKind } from "../types.ts";

type JSONPointer = string;

export type CfSchema = {
  typeName: string;
  description: string;
  primaryIdentifier: JSONPointer[];
  sourceUrl?: string;
  documentationUrl?: string;
  replacementStrategy?: "create_then_delete" | "delete_then_create";
  taggable?: boolean;
  tagging?: {
    taggable: boolean;
    tagOnCreate?: boolean;
    tagUpdatable?: boolean;
    cloudFormationSystemTags?: boolean;
    tagProperty?: string;
  };
  handlers?: { [key in CfHandlerKind]?: CfHandler };
  remote?: unknown;
  definitions?: Record<string, CfProperty>;
  properties: Record<string, CfProperty>;
  readOnlyProperties?: JSONPointer[];
  writeOnlyProperties?: JSONPointer[];
  conditionalCreateOnlyProperties?: JSONPointer[];
  nonPublicProperties?: JSONPointer[];
  nonPublicDefinitions?: JSONPointer[];
  createOnlyProperties?: JSONPointer[];
  deprecatedProperties?: JSONPointer[];
  additionalIdentifiers?: JSONPointer[];
  resourceLink?: {
    $comment: JSONSchema.Interface["$comment"];
    templateUri: string;
    mappings: Record<string, JSONPointer>;
  };
  propertyTransform?: Record<string, string>;
};

export type CfDb = Record<string, CfSchema>;
