/**
 * Shared utilities for resolving component references in subscriptions.
 *
 * This module provides functions to resolve component names and search queries
 * to component IDs, used by both templates and the component update command.
 *
 * @module
 */

import type { Logger } from "@logtape/logtape";
import type {
  ComponentSearchResult,
  SearchV1Response,
} from "@systeminit/api-client";

/**
 * Search function type - allows callers to provide their own search implementation
 * (e.g., with caching in templates, or direct API calls in updates)
 */
export type SearchFunction = (
  workspaceId: string,
  changeSetId: string,
  query: string,
) => Promise<SearchV1Response>;

/**
 * Get component function type - for fetching full component data when needed
 */
export type GetComponentFunction = (
  workspaceId: string,
  changeSetId: string,
  componentId: string,
) => Promise<{ component: { schemaId: string; name: string } }>;

/**
 * Get schema name function type - for enriching error messages
 */
export type GetSchemaNameFunction = (
  workspaceId: string,
  changeSetId: string,
  schemaId: string,
) => Promise<string>;

/**
 * Resolves a component reference (ULID or name) to a component ID.
 *
 * If the reference is already a ULID, returns it unchanged.
 * If it's a name, searches for the component and returns its ID.
 *
 * @param componentRef - Component reference (ULID or name)
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param searchFn - Function to execute search queries
 * @param logger - Logger instance
 * @returns Resolved component ID (ULID)
 * @throws Error if component not found or name is ambiguous
 */
export async function resolveComponentReference(
  componentRef: string,
  workspaceId: string,
  changeSetId: string,
  searchFn: SearchFunction,
  logger: Logger,
): Promise<string> {
  // Check if component is a ULID (26 chars, alphanumeric)
  // ULIDs use Crockford's base32: 0-9 and A-Z excluding I, L, O, U
  const isUlid = /^[0-9A-HJKMNP-TV-Z]{26}$/i.test(componentRef);

  if (isUlid) {
    logger.debug("Component is a ULID, no resolution needed: {id}", {
      id: componentRef,
    });
    return componentRef;
  }

  // Component is a name, need to search for it
  const searchQuery = `name: "${componentRef}"`;
  logger.debug("Searching for component by name: {name}", {
    name: componentRef,
  });

  const searchResult = await searchFn(workspaceId, changeSetId, searchQuery);
  const foundComponents = searchResult.components;

  if (foundComponents.length === 0) {
    throw new Error(`No component found with name: "${componentRef}"`);
  }

  if (foundComponents.length > 1) {
    const componentNames = foundComponents.map((c: { name: string }) =>
      `  - ${c.name}`
    ).join("\n");
    throw new Error(
      `Multiple components found with name "${componentRef}". Use component ID or search query instead.\n` +
        `Found:\n${componentNames}`,
    );
  }

  const componentId = foundComponents[0].id;
  logger.debug("Resolved component '{name}' to ID: {id}", {
    name: componentRef,
    id: componentId,
  });

  return componentId;
}

/**
 * Resolves a search query to a component ID.
 *
 * Executes the search and returns the ID of the first (and only) matching component.
 * Throws an error if zero or multiple components are found.
 *
 * @param query - Search query string
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param searchFn - Function to execute search queries
 * @param logger - Logger instance
 * @param getComponentFn - Optional function to fetch component details for error messages
 * @param getSchemaNameFn - Optional function to get schema names for error messages
 * @returns Resolved component ID (ULID)
 * @throws Error if no components found or multiple components found
 */
export async function resolveSearchQuery(
  query: string,
  workspaceId: string,
  changeSetId: string,
  searchFn: SearchFunction,
  logger: Logger,
  getComponentFn?: GetComponentFunction,
  getSchemaNameFn?: GetSchemaNameFunction,
): Promise<string> {
  logger.debug("Searching for component with query: {query}", { query });

  const searchResult = await searchFn(workspaceId, changeSetId, query);
  const foundComponents = searchResult.components;

  if (foundComponents.length === 0) {
    throw new Error(`No components found for search query: "${query}"`);
  }

  if (foundComponents.length > 1) {
    // If we have functions to enrich error message, use them
    if (getComponentFn && getSchemaNameFn) {
      const componentDetails: string[] = [];
      for (const comp of foundComponents) {
        try {
          const compResult = await getComponentFn(
            workspaceId,
            changeSetId,
            comp.id,
          );
          const schemaName = await getSchemaNameFn(
            workspaceId,
            changeSetId,
            compResult.component.schemaId,
          );
          componentDetails.push(`  - ${schemaName}: ${comp.name}`);
        } catch {
          componentDetails.push(`  - (unknown schema): ${comp.name}`);
        }
      }

      throw new Error(
        `Search returned ${foundComponents.length} components. Please refine your query to return exactly one.\n` +
          `Found:\n${componentDetails.join("\n")}`,
      );
    } else {
      // Simple error message with schema info from search results
      const componentDetails = foundComponents.map((c: ComponentSearchResult) =>
        `  - ${c.schema.name}: ${c.name}`
      ).join("\n");
      throw new Error(
        `Search returned ${foundComponents.length} components. Please refine your query to return exactly one.\n` +
          `Found:\n${componentDetails}`,
      );
    }
  }

  const foundComponent = foundComponents[0];
  logger.debug("Found component: {name} (ID: {id})", {
    name: foundComponent.name,
    id: foundComponent.id,
  });

  return foundComponent.id;
}
