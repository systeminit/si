import { CreatePropArgs, OnlyProperties } from "../spec/props.ts";
import type { JSONSchema } from "./draft_07.ts";
import type { Extend } from "../extend.ts";

type JSONPointer = string;

const CF_PROPERTY_TYPES = [
  "boolean",
  "string",
  "number",
  "integer",
  "object",
  "array",
  "json",
] as const;
export type CfPropertyType = typeof CF_PROPERTY_TYPES[number];

export type CfProperty =
  | Extend<CfBooleanProperty, { type: "boolean" }>
  | Extend<CfStringProperty, { type: "string" }>
  | Extend<CfNumberProperty, { type: "number" }>
  | Extend<CfIntegerProperty, { type: "integer" }>
  | Extend<CfArrayProperty, { type: "array" }>
  | CfObjectProperty // We may infer object-ness if type is undefined but other props are there
  | Omit<JSONSchema.String, "type"> & { type: "json" }
  | CfMultiTypeProperty
  // Then we have this mess of array typed properties
  | Extend<JSONSchema.Interface, {
    properties?: Record<string, CfProperty>;
    type: ["string", CfPropertyType] | [
      CfPropertyType,
      "string",
    ];
  }>;

export type CfBooleanProperty = JSONSchema.Boolean;

export type CfStringProperty = JSONSchema.String;

export type CfNumberProperty = JSONSchema.Number & {
  format?: string;
};

export type CfIntegerProperty = JSONSchema.Integer & {
  format?: string;
};

export type CfArrayProperty = Extend<JSONSchema.Array, {
  // For properties of type array, defines the data structure of each array item.
  // Contains a single schema. A list of schemas is not allowed.
  items: CfProperty;
  // For properties of type array, set to true to specify that the order in which array items are specified must be honored, and that changing the order of the array will indicate a change in the property.
  // The default is true.
  insertionOrder?: boolean;
}>;

export type CfObjectProperty = Extend<JSONSchema.Object, {
  properties?: Record<string, CfProperty>;
  // e.g. patternProperties: { "^[a-z]+": { type: "string" } }
  patternProperties?: Record<string, CfProperty>;
  // Any properties that are required if this property is specified.
  dependencies?: Record<string, string[]>;
  oneOf?: CfObjectProperty[];
  anyOf?: CfObjectProperty[];
  allOf?: CfObjectProperty[];
}>;

type CfMultiTypeProperty =
  & Pick<JSONSchema.Interface, "$ref" | "$comment" | "title" | "description">
  & {
    type?: undefined;
    oneOf?: CfProperty[];
    allOf?: CfProperty[];
    anyOf?: CfProperty[];
  };

type StringPair =
  | ["string", "object"]
  | ["string", "boolean"]
  | ["string", "number"]
  | ["string", "integer"]
  | ["object", "string"]
  | ["boolean", "string"]
  | ["number", "string"]
  | ["string", "array"]
  | ["integer", "string"];

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
  handlers?: Record<CfHandlerKind, CfHandler>;
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
    "$comment": JSONSchema.Interface["$comment"];
    templateUri: string;
    mappings: Record<string, JSONPointer>;
  };
  propertyTransform?: Record<string, string>;
};

export type CfHandlerKind = "create" | "read" | "update" | "delete" | "list";
export type CfHandler = {
  permissions: string[];
  timeoutInMinutes: number;
};

export type CfDb = Record<string, CfSchema>;

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

export type HDB = Record<string, HetznerSchema>;

export type HQueue = {
  superSchema: HetznerSchema;
  primaryIdentifier: JSONPointer[];
  onlyProperties: OnlyProperties;
  queue: CreatePropArgs[];
};

export type SuperSchema = HetznerSchema | CfSchema;

export type CategoryFn = ({ typeName }: SuperSchema) => string;
