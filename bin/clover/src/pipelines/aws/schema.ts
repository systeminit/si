import type { JSONSchema } from "../draft_07.ts";
import type {
  CfHandler,
  CfHandlerKind,
  CfProperty,
  JSONPointer,
  SuperSchema,
} from "../types.ts";

export interface CfSchema extends SuperSchema {
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
  primaryIdentifier: JSONPointer[];
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
  /**
   * Explicit mapping of writeOnly property names to their secret kinds.
   * Required for any writeOnly properties that should be treated as secrets.
   *
   * Example:
   * ```typescript
   * secretKinds: {
   *   credential: "AWS Credential",
   *   apiKey: "API Key",
   * }
   * ```
   */
  secretKinds?: Record<string, string>;
}

export type CfDb = Record<string, CfSchema>;
