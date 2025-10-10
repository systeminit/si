import type { JSONSchema } from "./draft_07.ts";
import type { HetznerSchema } from "./hetzner/schema.ts";
import type { AzureSchema } from "./azure/schema.ts";
import type { Extend } from "../extend.ts";
import { ActionFuncSpecKind } from "../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../spec/funcs.ts";
import type { CfSchema } from "./aws/schema.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import {
  ExpandedPropSpec,
  ExpandedPropSpecFor,
  OnlyProperties,
} from "../spec/props.ts";

const CF_PROPERTY_TYPES = [
  "boolean",
  "string",
  "number",
  "integer",
  "object",
  "array",
  "json",
] as const;
export type CfPropertyType = (typeof CF_PROPERTY_TYPES)[number];

export type CfProperty =
  | Extend<CfBooleanProperty, { type: "boolean" }>
  | Extend<CfStringProperty, { type: "string" }>
  | Extend<CfNumberProperty, { type: "number" }>
  | Extend<CfIntegerProperty, { type: "integer" }>
  | Extend<CfArrayProperty, { type: "array" }>
  | CfObjectProperty // We may infer object-ness if type is undefined but other props are there
  | (Omit<JSONSchema.String, "type"> & { type: "json" })
  | CfMultiTypeProperty
  // Then we have this mess of array typed properties
  | Extend<
      JSONSchema.Interface,
      {
        properties?: Record<string, CfProperty>;
        type: ["string", CfPropertyType] | [CfPropertyType, "string"];
      }
    >;

export type CfBooleanProperty = JSONSchema.Boolean;

export type CfStringProperty = JSONSchema.String;

export type CfNumberProperty = JSONSchema.Number & {
  format?: string;
};

export type CfIntegerProperty = JSONSchema.Integer & {
  format?: string;
};

export type CfArrayProperty = Extend<
  JSONSchema.Array,
  {
    // For properties of type array, defines the data structure of each array item.
    // Contains a single schema. A list of schemas is not allowed.
    items: CfProperty;
    // For properties of type array, set to true to specify that the order in which array items are specified must be honored, and that changing the order of the array will indicate a change in the property.
    // The default is true.
    insertionOrder?: boolean;
  }
>;

export type CfObjectProperty = Extend<
  JSONSchema.Object,
  {
    properties?: Record<string, CfProperty>;
    // e.g. patternProperties: { "^[a-z]+": { type: "string" } }
    patternProperties?: Record<string, CfProperty>;
    // Any properties that are required if this property is specified.
    dependencies?: Record<string, string[]>;
    oneOf?: CfObjectProperty[];
    anyOf?: CfObjectProperty[];
    allOf?: CfObjectProperty[];
  }
>;

type CfMultiTypeProperty = Pick<
  JSONSchema.Interface,
  "$ref" | "$comment" | "title" | "description"
> & {
  type?: undefined;
  oneOf?: CfProperty[];
  allOf?: CfProperty[];
  anyOf?: CfProperty[];
};

export type CfHandlerKind = "create" | "read" | "update" | "delete" | "list";
export type CfHandler = {
  permissions: string[];
  timeoutInMinutes: number;
};

export type { CfDb, CfSchema } from "./aws/schema.ts";

export type SuperSchema = HetznerSchema | CfSchema | AzureSchema;

export type CategoryFn = ({ typeName }: SuperSchema) => string;

/**
 * Provider-specific functions that must be implemented for each provider.
 * These functions handle provider-specific logic for transforming schemas
 * into the unified spec format.
 */
export interface ProviderFunctions {
  /**
   * Creates a documentation link for a schema, definition, or property.
   * @param schema - The schema to create a link for
   * @param defName - Optional definition name within the schema
   * @param propName - Optional property name within the schema or definition
   * @returns A URL string pointing to the provider's documentation
   */
  createDocLink: (
    schema: SuperSchema,
    defName?: string,
    propName?: string,
  ) => string;

  /**
   * Determines the category for a schema (e.g., "AWS::EC2", "Hetzner::Cloud").
   * @param schema - The schema to categorize
   * @returns A category string used for organizing resources
   */
  getCategory: (schema: SuperSchema) => string;
}

/**
 * Provider-specific func spec definitions that define the actions, code generation,
 * management, and qualification functions available for a provider.
 */
export interface ProviderFuncSpecs {
  /**
   * Action func specs (create, update, delete, refresh, etc.)
   * Maps function names to their specs with action kinds
   */
  actions: Record<string, FuncSpecInfo & { actionKind: ActionFuncSpecKind }>;

  /**
   * Code generation func specs
   * Maps function names to their specs
   */
  codeGeneration: Record<string, FuncSpecInfo>;

  /**
   * Management func specs (discover, import, etc.)
   * Maps function names to their specs with handler requirements
   */
  management: Record<string, FuncSpecInfo & { handlers: CfHandlerKind[] }>;

  /**
   * Qualification func specs
   * Maps function names to their specs
   */
  qualification: Record<string, FuncSpecInfo>;
}

/**
 * Options passed to provider pipeline functions
 */
export interface PipelineOptions {
  forceUpdateExistingPackages?: boolean;
  moduleIndexUrl: string;
  docLinkCache: string;
  inferred: string;
  services?: string[];
}

/**
 * Visual and descriptive metadata for a provider
 */
export interface ProviderMetadata {
  /**
   * Display color for UI elements (hex code, e.g., "#FF9900")
   * Defaults to "#FF9900" (AWS orange) if not specified
   */
  color?: string;

  /**
   * Human-readable display name for the provider
   * If not specified, the provider name will be capitalized
   */
  displayName?: string;

  /**
   * Brief description of the provider
   */
  description?: string;
}

/**
 * Context passed to property normalization hooks
 */
export interface PropertyNormalizationContext {
  /**
   * Path to this property from the root (e.g., ["root", "domain", "propertyName"])
   */
  propPath: string[];

  /**
   * The schema containing this property
   */
  schema: SuperSchema;

  /**
   * The parent property, if this property is nested
   */
  parentProp?: ExpandedPropSpecFor["object" | "array" | "map"];
}

/**
 * Function type for property-level overrides
 */
export type PropOverrideFn = (
  prop: ExpandedPropSpec,
  spec: ExpandedPkgSpec,
) => void;

/**
 * Function type for schema-level overrides
 */
export type SchemaOverrideFn = (spec: ExpandedPkgSpec) => void;

/**
 * Configuration object for a provider that groups all provider-specific
 * functionality and metadata. This serves as the single source of truth
 * for how a provider transforms its schemas into the unified spec format.
 */
export interface ProviderConfig {
  /**
   * Unique identifier for this provider (e.g., "aws", "hetzner", "dummy")
   */
  name: string;

  /**
   * Provider-specific functions for documentation, categorization, and normalization
   */
  functions: ProviderFunctions;

  /**
   * Provider-specific func spec definitions
   */
  funcSpecs: ProviderFuncSpecs;

  /**
   * Provider-specific schema loading and transformation function.
   * This function should load the provider's schemas from their source format
   * and transform them into ExpandedPkgSpec[] format, applying any
   * provider-specific pipeline steps.
   * @param options - Pipeline options including paths and filters
   * @returns Promise of array of expanded package specs
   */
  loadSchemas: (options: PipelineOptions) => Promise<ExpandedPkgSpec[]>;

  /**
   * Function to fetch/update the provider's schema from its source.
   * Should download or generate the provider's schema file and save it to src/provider-schemas/
   */
  fetchSchema: () => Promise<void>;

  /**
   * Visual and descriptive metadata for the provider
   * Used for UI display and documentation
   */
  metadata?: ProviderMetadata;

  /**
   * Function to normalize a single property before it's processed.
   * Called during prop creation for every property in the schema.
   * This allows providers to handle schema-specific quirks (e.g., OpenAPI oneOf,
   * CloudFormation multi-type arrays) before the property is transformed into
   * an SI prop spec.
   *
   * @param prop - The raw property from the provider's schema
   * @param context - Additional context (property path, parent schema, etc.)
   * @returns Normalized CfProperty that can be processed by createPropFromCf
   */
  normalizeProperty: (
    prop: CfProperty,
    context: PropertyNormalizationContext,
  ) => CfProperty;

  /**
   * Function to determine if a child property is required.
   *
   * This is provider-specific because different schema formats store required
   * information differently (e.g., Hetzner uses a Set at schema level,
   * CloudFormation uses an array in the parent property's definition).
   *
   * @param schema - The schema containing the property
   * @param parentProp - The parent property (if any)
   * @param childName - The name of the child property
   * @returns true if the property is required
   */
  isChildRequired: (
    schema: SuperSchema,
    parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
    childName: string,
  ) => boolean;

  /**
   * Optional function to parse raw provider schema(s) into normalized SuperSchema(s).
   * This is where provider-specific parsing and inference happens (e.g., parsing
   * OpenAPI specs, inferring property classifications from HTTP methods).
   *
   * If not provided, the provider is expected to handle schema parsing elsewhere
   * (e.g., AWS uses a pre-built CloudFormation database).
   *
   * @param rawSchema - The raw schema in provider's native format
   * @returns Array of normalized SuperSchemas ready for makeModule
   */
  parseRawSchema?: (rawSchema: unknown) => SuperSchema[];

  /**
   * Function to extract OnlyProperties classification from a schema.
   * This determines which properties are createOnly, readOnly, writeOnly,
   * and what the primaryIdentifier is.
   *
   * @param schema - The normalized SuperSchema
   * @returns Classification of properties by mutability
   */
  classifyProperties: (schema: SuperSchema) => OnlyProperties;

  /**
   * Required provider-specific asset and property overrides
   * Applied during the pipeline after basic spec generation
   */
  overrides: {
    propOverrides: Record<
      string,
      Record<string, PropOverrideFn | PropOverrideFn[]>
    >;
    schemaOverrides: Map<string, SchemaOverrideFn | SchemaOverrideFn[]>;
  };
}

/**
 * Registry of all available providers.
 * To add a new provider, simply add its config to this registry.
 */
export const PROVIDER_REGISTRY: Record<string, ProviderConfig> = {};
