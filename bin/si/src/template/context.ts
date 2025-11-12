import type { Logger } from "@logtape/logtape";
import { Context } from "../context.ts";
import { basename } from "@std/path";
import { z } from "zod";
import type { SubscriptionInputType } from "../template.ts";
import {
  ComponentsApi,
  type ComponentViewV1,
  type Configuration,
  type GetComponentV1Response,
  type GetSchemaV1Response,
  SchemasApi,
  SearchApi,
  type SearchV1Response,
} from "@systeminit/api-client";
import { getHeadChangeSetId } from "../si_client.ts";
import {
  deepEqual,
  extractSubscription,
  isSubscription,
} from "./attribute_diff.ts";
import { generateULID } from "./ulid.ts";
import {
  cachedGetComponent,
  cachedGetSchema,
  cachedGetSchemaIdByName,
} from "../cache.ts";
import {
  resolveComponentReference,
  resolveSearchQuery,
  type SearchFunction,
} from "../subscription_resolver.ts";

/**
 * Configuration options for creating a TemplateContext and running a template.
 *
 * These options control template execution behavior, including how the template
 * identifies itself, where to load/cache data, and whether to perform a dry run.
 */
export interface TemplateContextOptions {
  /** Invocation key for idempotency control - templates with the same key are treated as the same invocation */
  key: string;
  /** Optional path to input data file (JSON or YAML) for template inputs */
  input?: string;
  /** Optional path to baseline data file (JSON or YAML) to load cached baseline instead of querying API */
  baseline?: string;
  /** Optional path to cache baseline results (JSON or YAML) for faster subsequent runs */
  cacheBaseline?: string;
  /** Optional flag to exit after writing baseline cache without executing the template */
  cacheBaselineOnly?: boolean;
  /** Optional flag to show planned changes without executing them (dry run mode) */
  dryRun?: boolean;
}

/**
 * A name pattern for transforming component names using regular expression matching.
 *
 * Patterns are applied sequentially in array order. The replacement string supports
 * EJS templating with access to `inputs` (validated input data) and `c` (TemplateContext).
 *
 * @example
 * ```ts
 * c.namePattern([
 *   { pattern: /^dev-/, replacement: "prod-" },
 *   { pattern: /-v(\d+)$/, replacement: "-v<%= inputs.version %>" }
 * ]);
 * ```
 */
export interface NamePattern {
  /** Regular expression to match against component names */
  pattern: RegExp;
  /** Replacement string (supports EJS templates with `inputs` and `c` variables) */
  replacement: string;
}

/**
 * A focused component type that stores only the essential properties
 * needed for template baseline and working set operations.
 */
export interface TemplateComponent {
  id: string;
  schemaId: string;
  name: string;
  resourceId: string;
  /** Only attributes with paths starting with /si, /domain, or /secrets */
  attributes: { [key: string]: unknown };
}

/**
 * Predicate function for matching component attributes based on path, value, and component properties.
 *
 * Used by {@link TemplateContext.setAttribute}, {@link TemplateContext.deleteAttribute},
 * and {@link TemplateContext.setSiblingAttribute} to selectively operate on attributes.
 *
 * @param path - The attribute path (e.g., "/domain/instanceType")
 * @param value - The current value at that path
 * @param component - The component being evaluated
 * @returns true if the attribute matches the criteria, false otherwise
 *
 * @example
 * ```ts
 * // Match all tags with "temp-" prefix
 * const predicate: AttributePredicate = (path, value) =>
 *   path.startsWith("/si/tags/temp-");
 * ```
 */
export type AttributePredicate = (
  path: string,
  value: unknown,
  component: TemplateComponent,
) => boolean;

/**
 * Predicate function for matching attribute values.
 *
 * Used by {@link TemplateContext.setSiblingAttribute} to filter based on attribute values.
 *
 * @param value - The attribute value to evaluate
 * @returns true if the value matches the criteria, false otherwise
 *
 * @example
 * ```ts
 * // Match values that are arrays with more than 3 elements
 * const predicate: ValuePredicate = (value) =>
 *   Array.isArray(value) && value.length > 3;
 * ```
 */
export type ValuePredicate = (
  value: unknown,
) => boolean;

/**
 * Transform function that modifies the working set of components.
 *
 * This is the core user-defined function in a template where custom logic is implemented.
 * It receives the working set (post-name-transformation) and optional validated input data,
 * and must return the modified working set.
 *
 * @template TInput - The type of validated input data (inferred from Zod schema)
 * @param workingSet - Array of components to transform (can be modified in place or replaced)
 * @param inputData - Optional validated input data from the input file
 * @returns The transformed working set (can be the same array or a new one)
 *
 * @example
 * ```ts
 * c.transform(async (workingSet, inputs) => {
 *   // Filter to only EC2 instances
 *   const ec2Only = workingSet.filter(comp =>
 *     comp.name.includes("ec2")
 *   );
 *
 *   // Modify attributes
 *   for (const comp of ec2Only) {
 *     c.setAttribute(comp, "/domain/region", inputs.region);
 *   }
 *
 *   return ec2Only;
 * });
 * ```
 */
export type TransformFunction<TInput = unknown> = (
  workingSet: TemplateComponent[],
  inputData?: TInput,
) => TemplateComponent[] | Promise<TemplateComponent[]>;

// Re-export ComponentViewV1 for convenience
export type { ComponentViewV1 };

/**
 * Filter component attributes to only include paths starting with
 * /si, /domain, or /secrets.
 *
 * @param attributes - Full attributes object from ComponentViewV1
 * @returns Filtered attributes object
 */
export function filterComponentAttributes(
  attributes: { [key: string]: unknown },
): { [key: string]: unknown } {
  const filtered: { [key: string]: unknown } = {};

  for (const [key, value] of Object.entries(attributes)) {
    if (
      key.startsWith("/si") || key.startsWith("/domain") ||
      key.startsWith("/secrets")
    ) {
      filtered[key] = value;
    }
  }

  return filtered;
}

/**
 * Convert a full ComponentViewV1 to a TemplateComponent with only
 * essential properties and filtered attributes.
 *
 * @param component - Full component from API
 * @returns Filtered TemplateComponent
 */
export function componentViewToTemplateComponent(
  component: ComponentViewV1,
): TemplateComponent {
  return {
    id: component.id,
    schemaId: component.schemaId,
    name: component.name,
    resourceId: component.resourceId,
    attributes: filterComponentAttributes(component.attributes || {}),
  };
}

/**
 * The main context object provided to templates for configuring and executing template logic.
 *
 * TemplateContext provides methods for:
 * - Configuring template behavior (name, search, inputs, transforms)
 * - Accessing and modifying component data (baseline, working set)
 * - Interacting with the System Initiative API (schemas, components, subscriptions)
 * - Manipulating component attributes and creating new components
 *
 * This class is instantiated by the template runtime and passed to user template files.
 * Users interact with it through the exported `c` variable in their templates.
 *
 * @template TInputSchema - The Zod schema type for validating template inputs
 *
 * @example
 * ```ts
 * // In a template file
 * export default async function(c: TemplateContext) {
 *   c.search(["name:myapp-*"]);
 *   c.namePattern([{ pattern: /^myapp-/, replacement: "yourapp-" }]);
 *
 *   c.transform(async (workingSet) => {
 *     // Custom logic here
 *     return workingSet;
 *   });
 * }
 * ```
 */
export class TemplateContext<TInputSchema extends z.ZodTypeAny = z.ZodTypeAny> {
  public readonly logger: Logger;
  private readonly ctx: Context;
  private _name: string;
  private _invocationKey: string;
  private _changeSet: string;
  private _search: string[];
  private _namePattern: NamePattern[] | undefined;
  private _inputs: TInputSchema | undefined;
  private _inputData: z.infer<TInputSchema> | undefined;
  private _transform: TransformFunction<z.infer<TInputSchema>> | undefined;
  private _baseline: TemplateComponent[] | undefined;
  private _workingSet: TemplateComponent[] | undefined;
  private _schemaCache: Map<string, GetSchemaV1Response>;
  private _searchCache: Map<string, SearchV1Response>;
  private _componentCache: Map<string, GetComponentV1Response>;
  private _headChangeSetId: string | undefined;

  constructor(templatePath: string, options: TemplateContextOptions) {
    this.ctx = Context.instance();
    this.logger = this.ctx.logger;

    this._name = basename(templatePath, ".ts");
    this._invocationKey = options.key;
    this._changeSet = `${this._name}-${this._invocationKey}`;
    this._search = [];
    this._namePattern = undefined;
    this._inputs = undefined;
    this._inputData = undefined;
    this._transform = undefined;
    this._baseline = undefined;
    this._workingSet = undefined;
    this._schemaCache = new Map();
    this._searchCache = new Map();
    this._componentCache = new Map();
  }

  /**
   * Get or set the template name.
   *
   * @param newName - Optional new name to set
   * @returns The current name if no argument provided, otherwise void
   */
  name(newName?: string): string | void {
    if (newName !== undefined) {
      this.logger.debug(`Setting Name: {newName}`, { newName });
      this._name = newName;
    } else {
      return this._name;
    }
  }

  /**
   * Get or set the changeSet name. The default is the template file name, minus
   * the extension, plus the invocation key from the command line arguments.
   *
   * @param newChangeSet - Optional new change set name to set
   * @returns The current name if no argument provided, otherwise void
   */
  changeSet(newChangeSet?: string): string | void {
    if (newChangeSet !== undefined) {
      this.logger.debug(`Setting Change Set: {newChangeSet}`, { newChangeSet });
      this._changeSet = newChangeSet;
    } else {
      return this._changeSet;
    }
  }

  /**
   * Get the invocation key.
   *
   * @returns The invocation key
   */
  invocationKey(): string {
    return this._invocationKey;
  }

  /**
   * Get or set the search strings.
   *
   * @param newSearch - Optional new search array to set
   * @returns The current search array if no argument provided, otherwise void
   */
  search(newSearch?: string[]): string[] | void {
    if (newSearch !== undefined) {
      this.logger.debug(`Setting Search: {newSearch}`, { newSearch });
      this._search = newSearch;
    } else {
      return this._search;
    }
  }

  /**
   * Get or set the name patterns for transforming component names.
   * Patterns are applied sequentially in array order.
   */
  namePattern(): NamePattern[] | undefined;
  namePattern(newPatterns: NamePattern[]): void;
  namePattern(newPatterns?: NamePattern[]): NamePattern[] | undefined | void {
    if (newPatterns !== undefined) {
      this.logger.debug(
        `Setting Name Patterns: ${newPatterns.length} pattern(s)`,
      );
      for (const pattern of newPatterns) {
        this.logger.debug(
          `  Pattern: ${pattern.pattern.source} -> ${pattern.replacement}`,
        );
      }
      this._namePattern = newPatterns;
    } else {
      return this._namePattern;
    }
  }

  /**
   * Get or set the input schema for validating template inputs.
   */
  inputs(): TInputSchema | undefined;
  inputs(newSchema: TInputSchema): void;
  inputs(newSchema?: TInputSchema): TInputSchema | undefined | void {
    if (newSchema !== undefined) {
      const jsonSchema = z.toJSONSchema(newSchema);
      this.logger.debug(`Setting Input Schema:\n{*}`, jsonSchema);
      this._inputs = newSchema;
    } else {
      return this._inputs;
    }
  }

  /**
   * Get or set the input data for the template.
   */
  inputData(): z.infer<TInputSchema> | undefined;
  inputData(data: z.infer<TInputSchema>): void;
  inputData(
    data?: z.infer<TInputSchema>,
  ): z.infer<TInputSchema> | undefined | void {
    if (data !== undefined) {
      this.logger.debug(`Setting Input Data with {count} keys`, {
        count: typeof data === "object" && data !== null
          ? Object.keys(data).length
          : 0,
      });
      this._inputData = data;
    } else {
      return this._inputData;
    }
  }

  /**
   * Get or set the transformation function for the working set of components.
   */
  transform(): TransformFunction<z.infer<TInputSchema>> | undefined;
  transform(fn: TransformFunction<z.infer<TInputSchema>>): void;
  transform(
    fn?: TransformFunction<z.infer<TInputSchema>>,
  ): TransformFunction<z.infer<TInputSchema>> | undefined | void {
    if (fn !== undefined) {
      this.logger.debug(`Setting Transform Function:\n{fnSource}`, {
        fnSource: fn.toString(),
      });
      this._transform = fn;
    } else {
      return this._transform;
    }
  }

  /**
   * Get the System Initiative API client configuration.
   *
   * @returns The API client configuration, or undefined if not initialized
   */
  apiConfig(): Configuration | undefined {
    return this.ctx.apiConfig;
  }

  /**
   * Get the workspace ID extracted from the API token.
   *
   * @returns The workspace ID, or undefined if not initialized
   */
  workspaceId(): string | undefined {
    return this.ctx.workspaceId;
  }

  /**
   * Get the user ID extracted from the API token.
   *
   * @returns The user ID, or undefined if not initialized
   */
  userId(): string | undefined {
    return this.ctx.userId;
  }

  /**
   * Get the HEAD changeset ID with caching.
   * The first call fetches from the API, subsequent calls return the cached value.
   *
   * @returns The HEAD changeset ID
   * @throws Error if API configuration is not available or HEAD changeset is not found
   */
  async getHeadChangeSetId(): Promise<string> {
    if (this._headChangeSetId) {
      this.logger.trace(`HEAD changeset cache hit: {id}`, {
        id: this._headChangeSetId,
      });
      return this._headChangeSetId;
    }

    const apiConfig = this.apiConfig();
    const workspaceId = this.workspaceId();

    if (!apiConfig || !workspaceId) {
      throw new Error(
        "Cannot get HEAD changeset: API configuration not available",
      );
    }

    this.logger.debug(`Fetching HEAD changeset for workspace {workspaceId}`, {
      workspaceId,
    });
    const changeSetId = await getHeadChangeSetId(apiConfig, workspaceId);
    this._headChangeSetId = changeSetId;
    this.logger.debug(`Found HEAD changeset: {id} ("HEAD")`, {
      id: changeSetId,
    });

    return changeSetId;
  }

  /**
   * Get or set the baseline data.
   */
  baseline(): TemplateComponent[] | undefined;
  baseline(data: TemplateComponent[]): void;
  baseline(data?: TemplateComponent[]): TemplateComponent[] | undefined | void {
    if (data !== undefined) {
      this.logger.debug(`Setting Baseline: {count} components`, {
        count: data.length,
      });
      this._baseline = data;
    } else {
      return this._baseline;
    }
  }

  /**
   * Get or set the working set data.
   */
  workingSet(): TemplateComponent[] | undefined;
  workingSet(data: TemplateComponent[]): void;
  workingSet(
    data?: TemplateComponent[],
  ): TemplateComponent[] | undefined | void {
    if (data !== undefined) {
      this.logger.debug(`Setting Working Set: {count} components`, {
        count: data.length,
      });
      this._workingSet = data;
    } else {
      return this._workingSet;
    }
  }

  /**
   * Get the schema cache.
   *
   * @returns The schema cache Map
   */
  schemaCache(): Map<string, GetSchemaV1Response> {
    return this._schemaCache;
  }

  /**
   * Get schema name for a given schema ID, with caching.
   * Checks cache first, fetches from API if not cached.
   *
   * This is useful for logging and display purposes when you have a component's
   * schemaId but want to show a human-readable name like "AWS EC2 Instance".
   *
   * @param workspaceId - Workspace ID
   * @param changeSetId - Change set ID
   * @param schemaId - Schema ID to lookup
   * @returns The schema name, or the schemaId if lookup fails
   *
   * @example
   * ```ts
   * const component = workingSet[0];
   * const schemaName = await c.getSchemaName(
   *   c.workspaceId(),
   *   await c.getHeadChangeSetId(),
   *   component.schemaId
   * );
   * console.log(`Component is a ${schemaName}`);
   * // Output: "Component is a AWS EC2 Instance"
   * ```
   */
  async getSchemaName(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
  ): Promise<string> {
    try {
      const apiConfig = this.apiConfig();
      if (!apiConfig) {
        this.logger.warn(
          "Cannot fetch schema: API configuration not available",
        );
        return schemaId;
      }

      const schemasApi = new SchemasApi(apiConfig);
      const schema = await cachedGetSchema(
        this._schemaCache,
        schemasApi,
        this.logger,
        workspaceId,
        changeSetId,
        schemaId,
      );

      return schema.name;
    } catch (error) {
      this.logger.error(`Failed to fetch schema {schemaId}: {error}`, {
        schemaId,
        error: error instanceof Error ? error.message : String(error),
      });
      return schemaId;
    }
  }

  /**
   * Get schema ID for a given schema name, with caching.
   * Uses the findSchema API to efficiently look up a single schema by name.
   * Checks cache first, fetches from API if not cached.
   *
   * This is useful when you want to create or filter components based on their
   * schema type using human-readable names instead of IDs.
   *
   * @param workspaceId - Workspace ID
   * @param changeSetId - Change set ID
   * @param schemaName - Schema name to lookup (e.g., "AWS EC2 Instance")
   * @returns The schema ID
   * @throws Error if schema is not found or API call fails
   *
   * @example
   * ```ts
   * // Create a new component with a specific schema
   * const ec2SchemaId = await c.getSchemaIdByName(
   *   c.workspaceId(),
   *   await c.getHeadChangeSetId(),
   *   "AWS EC2 Instance"
   * );
   *
   * const newServer = c.newComponent("web-server-1", ec2SchemaId);
   * workingSet.push(newServer);
   * ```
   *
   * @example
   * ```ts
   * // Filter working set to only EC2 instances
   * const ec2SchemaId = await c.getSchemaIdByName(
   *   c.workspaceId(),
   *   await c.getHeadChangeSetId(),
   *   "AWS EC2 Instance"
   * );
   *
   * return workingSet.filter(comp => comp.schemaId === ec2SchemaId);
   * ```
   */
  async getSchemaIdByName(
    workspaceId: string,
    changeSetId: string,
    schemaName: string,
  ): Promise<string> {
    try {
      const apiConfig = this.apiConfig();
      if (!apiConfig) {
        throw new Error("Cannot fetch schema: API configuration not available");
      }

      const schemasApi = new SchemasApi(apiConfig);
      return await cachedGetSchemaIdByName(
        this._schemaCache,
        schemasApi,
        this.logger,
        workspaceId,
        changeSetId,
        schemaName,
      );
    } catch (error) {
      this.logger.error(`Failed to find schema "{schemaName}": {error}`, {
        schemaName,
        error: error instanceof Error ? error.message : String(error),
      });
      throw new Error(`Schema not found: ${schemaName}`);
    }
  }

  /**
   * Cached search that performs API search only once per unique query string.
   * Subsequent calls with the same query return the cached result.
   *
   * @param workspaceId - Workspace ID
   * @param changeSetId - Change set ID
   * @param query - Search query string
   * @returns The search result
   * @private
   */
  private async _cachedSearch(
    workspaceId: string,
    changeSetId: string,
    query: string,
  ): Promise<SearchV1Response> {
    // Check cache first
    const cacheKey = `${workspaceId}:${changeSetId}:${query}`;
    const cached = this._searchCache.get(cacheKey);
    if (cached) {
      this.logger.debug(`Search cache hit for query: {query}`, { query });
      return cached;
    }

    // Cache miss - perform API call
    this.logger.debug(`Search cache miss for query: {query}`, { query });
    const apiConfig = this.apiConfig();
    if (!apiConfig) {
      throw new Error("Cannot perform search: API configuration not available");
    }

    const searchApi = new SearchApi(apiConfig);
    const searchResult = await searchApi.search({
      workspaceId,
      changeSetId,
      q: query,
    });

    // Cache the result
    this._searchCache.set(cacheKey, searchResult.data);
    this.logger.debug(
      `Cached search result for query: {query} ({count} components found)`,
      {
        query,
        count: searchResult.data.components.length,
      },
    );

    return searchResult.data;
  }

  /**
   * Cached component fetch that retrieves full component data only once per unique component ID.
   * Subsequent calls with the same component ID return the cached result.
   *
   * @param workspaceId - Workspace ID
   * @param changeSetId - Change set ID
   * @param componentId - Component ID to fetch
   * @returns The component data response
   * @private
   */
  private async _cachedGetComponent(
    workspaceId: string,
    changeSetId: string,
    componentId: string,
  ): Promise<GetComponentV1Response> {
    const apiConfig = this.apiConfig();
    if (!apiConfig) {
      throw new Error(
        "Cannot fetch component: API configuration not available",
      );
    }

    const componentsApi = new ComponentsApi(apiConfig);
    return await cachedGetComponent(
      this._componentCache,
      componentsApi,
      this.logger,
      workspaceId,
      changeSetId,
      componentId,
    );
  }

  /**
   * Set a subscription on a component's attribute path.
   *
   * Subscriptions allow one component to receive values from another component's attributes.
   * You can subscribe using either a search query to find the source component, or by
   * directly referencing a component by name or ID.
   *
   * @param component - The component to set the subscription on
   * @param attributePath - The attribute path where the subscription should be set (e.g., "/domain/config")
   * @param subscription - The subscription specification (either search or $source)
   * @throws Error if search returns zero or multiple components, or if validation fails
   *
   * @example
   * ```ts
   * // Subscribe using a search query to find a component
   * await c.setSubscription(webServer, "/domain/databaseConnection", {
   *   kind: "search",
   *   query: 'name:"postgres-db"',
   *   path: "/domain/connectionString"
   * });
   *
   * // Subscribe using direct component reference by name
   * await c.setSubscription(webServer, "/domain/config", {
   *   kind: "$source",
   *   component: "config-service",
   *   path: "/domain/endpoint"
   * });
   *
   * // Subscribe using direct component reference by ID (ULID)
   * await c.setSubscription(webServer, "/domain/config", {
   *   kind: "$source",
   *   component: "01HQXYZ123ABC456DEF789GHJ0",
   *   path: "/domain/endpoint"
   * });
   *
   * // Subscribe with an optional transformation function
   * await c.setSubscription(loadBalancer, "/domain/servers", {
   *   kind: "search",
   *   query: 'schemaName:"AWS EC2 Instance"',
   *   path: "/domain/privateIp",
   *   func: "si:normalizeToArray"  // Convert single value to array
   * });
   * ```
   */
  async setSubscription(
    component: TemplateComponent,
    attributePath: string,
    subscription: SubscriptionInputType,
  ): Promise<void> {
    let componentId: string;

    if (subscription.kind === "$source") {
      // For ULIDs, we can skip API calls entirely
      const isUlid = /^[0-9A-HJKMNP-TV-Z]{26}$/i.test(subscription.component);

      if (isUlid) {
        componentId = subscription.component;
      } else {
        // Component is a name - need API access to resolve it
        const apiConfig = this.apiConfig();
        const workspaceId = this.workspaceId();

        if (!apiConfig || !workspaceId) {
          throw new Error(
            "Cannot set subscription: API configuration not available",
          );
        }

        const changeSetId = await this.getHeadChangeSetId();

        // Create search function wrapper that uses cached search
        const searchFn: SearchFunction = (
          workspaceId: string,
          changeSetId: string,
          query: string,
        ) => this._cachedSearch(workspaceId, changeSetId, query);

        // Direct $source subscription - resolve component name to ID
        componentId = await resolveComponentReference(
          subscription.component,
          workspaceId,
          changeSetId,
          searchFn,
          this.logger,
        );
      }

      // Check if subscription already matches
      const existingValue = component.attributes[attributePath];
      if (isSubscription(existingValue)) {
        const existingSub = extractSubscription(existingValue);
        const newSub = {
          component: componentId,
          path: subscription.path,
          func: subscription.func,
        };

        if (deepEqual(existingSub, newSub)) {
          this.logger.trace(
            `Subscription already set on {componentName} at {attributePath} -> {targetComponent}:{targetPath}, skipping`,
            {
              componentName: component.name,
              attributePath,
              targetComponent: componentId,
              targetPath: subscription.path,
            },
          );
          return; // Skip update, subscription is already correct
        }
      }

      // Set the subscription using the resolved component ID
      component.attributes[attributePath] = {
        $source: {
          component: componentId,
          path: subscription.path,
          ...(subscription.func && { func: subscription.func }),
        },
      };
      this.logger.debug(
        `Set $source subscription on {componentName} at {attributePath} -> {targetComponent}:{targetPath}`,
        {
          componentName: component.name,
          attributePath,
          targetComponent: componentId,
          targetPath: subscription.path,
        },
      );
    } else {
      // Search subscription - need API access
      const apiConfig = this.apiConfig();
      const workspaceId = this.workspaceId();

      if (!apiConfig || !workspaceId) {
        throw new Error(
          "Cannot set subscription: API configuration not available",
        );
      }

      const changeSetId = await this.getHeadChangeSetId();

      // Create search function wrapper that uses cached search
      const searchFn: SearchFunction = (
        workspaceId: string,
        changeSetId: string,
        query: string,
      ) => this._cachedSearch(workspaceId, changeSetId, query);

      // Search subscription - resolve query to component ID
      componentId = await resolveSearchQuery(
        subscription.query,
        workspaceId,
        changeSetId,
        searchFn,
        this.logger,
        this._cachedGetComponent.bind(this),
        this.getSchemaName.bind(this),
      );

      // Fetch full component data and verify attribute exists
      const componentResult = await this._cachedGetComponent(
        workspaceId,
        changeSetId,
        componentId,
      );

      const fullComponent = componentResult.component;

      // Check if the attribute path exists (skip for secrets as they may not be in attributes)
      if (
        !subscription.path.startsWith("/secrets") &&
        !(subscription.path in (fullComponent.attributes || {}))
      ) {
        const schemaName = await this.getSchemaName(
          workspaceId,
          changeSetId,
          fullComponent.schemaId,
        );
        throw new Error(
          `Attribute "${subscription.path}" not found on component "${fullComponent.name}" (schema: ${schemaName})`,
        );
      }

      // Check if subscription already exists with the same values
      const existingValue = component.attributes[attributePath];
      if (isSubscription(existingValue)) {
        const existingSub = extractSubscription(existingValue);
        const newSub = {
          component: componentId,
          path: subscription.path,
          func: subscription.func,
        };

        if (deepEqual(existingSub, newSub)) {
          this.logger.trace(
            `Subscription already set on {componentName} at {attributePath} -> {targetName}:{targetPath}, skipping`,
            {
              componentName: component.name,
              attributePath,
              targetName: fullComponent.name,
              targetPath: subscription.path,
            },
          );
          return; // Skip update, subscription is already correct
        }
      }

      // Set the subscription using the found component's ID
      component.attributes[attributePath] = {
        $source: {
          component: componentId,
          path: subscription.path,
          ...(subscription.func && { func: subscription.func }),
        },
      };

      this.logger.info(
        `Set subscription on {componentName} at {attributePath} -> {targetName}:{targetPath}`,
        {
          componentName: component.name,
          attributePath,
          targetName: fullComponent.name,
          targetPath: subscription.path,
        },
      );
    }
  }

  /**
   * Deletes attributes from a component's working set based on a matcher.
   *
   * @param component - The component to modify
   * @param matcher - String (exact path match), RegExp (partial match), or predicate function
   *
   * @example
   * // Delete exact attribute path
   * c.deleteAttribute(component, "/domain/oldConfig");
   *
   * @example
   * // Delete all attributes matching regex
   * c.deleteAttribute(component, /^\/domain\/temp/);
   *
   * @example
   * // Delete using predicate function
   * c.deleteAttribute(component, (path, value, comp) =>
   *   path.startsWith("/si/tags/") && value === "deprecated"
   * );
   */
  deleteAttribute(
    component: TemplateComponent,
    matcher: string | RegExp | AttributePredicate,
  ): void {
    const logger = Context.instance().logger;
    const deletedPaths: string[] = [];

    // Iterate through all attributes and collect matching paths
    for (const [path, value] of Object.entries(component.attributes)) {
      let shouldDelete = false;

      if (typeof matcher === "string") {
        // Exact path match
        shouldDelete = path === matcher;
      } else if (matcher instanceof RegExp) {
        // Partial regex match
        shouldDelete = matcher.test(path);
      } else if (typeof matcher === "function") {
        // Predicate function
        shouldDelete = matcher(path, value, component);
      }

      if (shouldDelete) {
        delete component.attributes[path];
        deletedPaths.push(path);
      }
    }

    // Log the deletions
    if (deletedPaths.length > 0) {
      logger.debug(
        `Deleted ${deletedPaths.length} attribute(s) from component "${component.name}": ${
          deletedPaths.join(", ")
        }`,
      );
    } else {
      logger.debug(
        `No attributes matched for deletion in component "${component.name}"`,
      );
    }
  }

  /**
   * Sets attributes on a component's working set based on a matcher.
   *
   * @param component - The component to modify
   * @param matcher - String (exact path match), RegExp (matches any key), or predicate function
   * @param value - The value to set for matching paths
   *
   * @example
   * // Set exact attribute path
   * c.setAttribute(component, "/domain/config", { key: "value" });
   *
   * @example
   * // Set all attributes matching regex
   * c.setAttribute(component, /^\/domain\/temp/, "temporary");
   *
   * @example
   * // Set using predicate function
   * c.setAttribute(component, (path, value, comp) =>
   *   path.startsWith("/si/tags/") && value === "old"
   * , "new");
   */
  setAttribute(
    component: TemplateComponent,
    matcher: string | RegExp | AttributePredicate,
    value: unknown,
  ): void {
    const logger = Context.instance().logger;
    const updatedPaths: string[] = [];

    if (typeof matcher === "string") {
      // Exact path match - just set the value directly
      component.attributes[matcher] = value;
      updatedPaths.push(matcher);
    } else {
      // For RegExp and predicate, iterate through existing attributes
      for (
        const [path, existingValue] of Object.entries(component.attributes)
      ) {
        let shouldSet = false;

        if (matcher instanceof RegExp) {
          // Partial regex match
          shouldSet = matcher.test(path);
        } else if (typeof matcher === "function") {
          // Predicate function
          shouldSet = matcher(path, existingValue, component);
        }

        if (shouldSet) {
          component.attributes[path] = value;
          updatedPaths.push(path);
        }
      }
    }

    // Log the updates
    if (updatedPaths.length > 0) {
      logger.debug(
        `Set ${updatedPaths.length} attribute(s) on component "${component.name}": ${
          updatedPaths.join(", ")
        }`,
      );
    } else {
      logger.debug(
        `No attributes matched for update in component "${component.name}"`,
      );
    }
  }

  /**
   * Sets a sibling attribute for array element properties that match the given criteria.
   * This is useful for updating related properties within the same array element,
   * such as changing a Tag's Value when its Key matches a certain pattern.
   *
   * @param component - The component to modify
   * @param keyMatcher - String (exact path match), RegExp (partial match), or predicate function to find reference attributes
   * @param valueMatcher - The value to match (deep equality), or a predicate function that returns true for matching values
   * @param siblingName - The name of the sibling property to set (e.g., "Value" to set /path/N/Value)
   * @param siblingValue - The value to set on the sibling attribute
   *
   * @example
   * // Find /domain/Tags/0/Key with value "Name" and set /domain/Tags/0/Value
   * c.setSiblingAttribute(w, "/domain/Tags/0/Key", "Name", "Value", "poop-canoe");
   *
   * @example
   * // Use regex to match any Tag Key and set Value based on predicate
   * c.setSiblingAttribute(w, /\/domain\/Tags\/\d+\/Key/, (v) => v.startsWith("env-"), "Value", "production");
   *
   * @example
   * // Use predicate for key matching with value check
   * c.setSiblingAttribute(
   *   w,
   *   (path) => path.includes("SecurityGroupIngress") && path.endsWith("/IpProtocol"),
   *   "tcp",
   *   "FromPort",
   *   443
   * );
   */
  setSiblingAttribute(
    component: TemplateComponent,
    keyMatcher: string | RegExp | AttributePredicate,
    valueMatcher: unknown | ValuePredicate,
    siblingName: string,
    siblingValue: unknown,
  ): void {
    const logger = Context.instance().logger;
    const updatedSiblings: string[] = [];

    // Pattern to extract array element path: /path/to/array/N/property
    const arrayElementPattern = /^(\/[^/]+(?:\/[^/]+)*\/\d+)\/(.+)$/;

    // Iterate through all attributes to find matches
    for (const [path, value] of Object.entries(component.attributes)) {
      // First, check if the key matches
      let keyMatches = false;

      if (typeof keyMatcher === "string") {
        keyMatches = path === keyMatcher;
      } else if (keyMatcher instanceof RegExp) {
        keyMatches = keyMatcher.test(path);
      } else if (typeof keyMatcher === "function") {
        keyMatches = keyMatcher(path, value, component);
      }

      if (!keyMatches) {
        continue;
      }

      // Second, check if the value matches
      let valueMatches = false;

      if (typeof valueMatcher === "function") {
        valueMatches = valueMatcher(value);
      } else {
        // Use deep equality check
        valueMatches = deepEqual(value, valueMatcher);
      }

      if (!valueMatches) {
        continue;
      }

      // Both key and value match - extract array element path
      const match = path.match(arrayElementPattern);

      if (!match) {
        logger.warn(
          `Path "${path}" matches criteria but is not an array element property. ` +
            `Expected format: /path/to/array/N/property. Skipping.`,
        );
        continue;
      }

      const [, elementPath, currentProperty] = match;
      const siblingPath = `${elementPath}/${siblingName}`;

      // Set the sibling attribute
      component.attributes[siblingPath] = siblingValue;
      updatedSiblings.push(`${path} -> ${siblingPath}`);

      logger.debug(
        `Set sibling attribute on component "${component.name}": ` +
          `${path} (${currentProperty}="${JSON.stringify(value)}") -> ` +
          `${siblingPath} = ${JSON.stringify(siblingValue)}`,
      );
    }

    // Log summary
    if (updatedSiblings.length > 0) {
      logger.debug(
        `Set ${updatedSiblings.length} sibling attribute(s) on component "${component.name}"`,
      );
    } else {
      logger.trace(
        "No attributes matched criteria for sibling update in component {name}",
        { name: component.name },
      );
    }
  }

  /**
   * Creates a deep copy of a component with a new name and ID.
   * All attributes including subscriptions are preserved as-is.
   * The copied component is NOT automatically added to the working set -
   * you must explicitly push it to the working set array.
   *
   * @param source - The component to copy
   * @param newName - The name for the copied component (must not be empty)
   * @returns A new TemplateComponent with a unique ID and the specified name
   * @throws Error if newName is empty or whitespace-only
   *
   * @example
   * // Create 10 server instances from a template
   * c.transform(async (workingSet, inputs) => {
   *   const template = workingSet.find(c => c.name === "server-template");
   *
   *   for (let i = 0; i < inputs.serverCount; i++) {
   *     const copy = c.copyComponent(template, `server-${i + 1}`);
   *     workingSet.push(copy);
   *   }
   *
   *   return workingSet;
   * });
   */
  copyComponent(
    source: TemplateComponent,
    newName: string,
  ): TemplateComponent {
    const logger = Context.instance().logger;

    // Validate newName
    if (!newName || newName.trim().length === 0) {
      throw new Error("Component name cannot be empty");
    }

    // Deep clone the source component
    const copy = structuredClone(source);

    // Generate new unique ID
    copy.id = generateULID();

    // Update name in both the name field and the /si/name attribute
    copy.name = newName;
    if (copy.attributes["/si/name"]) {
      copy.attributes["/si/name"] = newName;
    }

    // Mark as dynamically created with stable name identifier for idempotent matching
    copy.attributes["/si/tags/templateDynamicName"] = newName;

    logger.debug(
      `Copied component "${source.name}" (ID: ${source.id}) -> "${newName}" (ID: ${copy.id})`,
    );

    return copy;
  }

  /**
   * Creates a new component from scratch with the specified schema, name, and attributes.
   * The component is NOT automatically added to the working set -
   * you must explicitly push it to the working set array.
   *
   * @param schemaName - The schema name to use for the component (e.g., "AWS EC2 Instance")
   * @param name - The name for the new component (must not be empty)
   * @param attributes - Optional attributes to set on the component (default: {})
   * @returns A new TemplateComponent with a unique ID, resolved schema ID, and the specified name and attributes
   * @throws Error if name is empty/whitespace, schema is not found, or API configuration is unavailable
   *
   * @example
   * // Create a new EC2 instance component
   * c.transform(async (workingSet, inputs) => {
   *   const newServer = await c.newComponent(
   *     "AWS EC2 Instance",
   *     "web-server-1",
   *     { "/domain/instanceType": "t2.micro" }
   *   );
   *   workingSet.push(newServer);
   *
   *   return workingSet;
   * });
   */
  async newComponent(
    schemaName: string,
    name: string,
    attributes: { [key: string]: unknown } = {},
  ): Promise<TemplateComponent> {
    const logger = Context.instance().logger;

    // Validate name
    if (!name || name.trim().length === 0) {
      throw new Error("Component name cannot be empty");
    }

    // Get API configuration and workspace ID
    const apiConfig = this.apiConfig();
    const workspaceId = this.workspaceId();

    if (!apiConfig || !workspaceId) {
      throw new Error(
        "Cannot create component: API configuration not available",
      );
    }

    // Get the HEAD changeset ID (cached)
    const changeSetId = await this.getHeadChangeSetId();

    // Look up schema ID by name
    const schemaId = await this.getSchemaIdByName(
      workspaceId,
      changeSetId,
      schemaName,
    );

    // Generate unique ID for the new component
    const componentId = generateULID();

    // Create the component with auto-set defaults and user attributes
    const component = {
      id: componentId,
      schemaId: schemaId,
      name: name,
      resourceId: "", // New components don't have a resourceId yet
      attributes: {
        "/si/name": name,
        "/si/type": "component",
        "/si/tags/templateDynamicName": name,
        ...attributes, // User attributes come after defaults
      },
    };

    logger.debug(
      `Created new component "${name}" with schema "${schemaName}" (ID: ${componentId}, Schema ID: ${schemaId})`,
    );

    return component;
  }
}
