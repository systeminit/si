/**
 * Shared caching utilities for component and schema lookups.
 *
 * This module provides reusable caching functions that can be used across
 * template execution and component operations to avoid redundant API calls.
 */

import type {
  ComponentsApi,
  GetComponentV1Response,
  GetSchemaV1Response,
  SchemasApi,
} from "@systeminit/api-client";
import type { Logger } from "@logtape/logtape";

/**
 * Cached component fetch that retrieves full component data only once per unique component ID.
 * Subsequent calls with the same component ID return the cached result.
 *
 * @param cache - Map to store cached component responses
 * @param componentsApi - Initialized ComponentsApi instance
 * @param logger - Logger instance for debug/trace logging
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param componentId - Component ID to fetch
 * @returns The component data response
 */
export async function cachedGetComponent(
  cache: Map<string, GetComponentV1Response>,
  componentsApi: ComponentsApi,
  logger: Logger,
  workspaceId: string,
  changeSetId: string,
  componentId: string,
): Promise<GetComponentV1Response> {
  // Check cache first
  const cacheKey = `${workspaceId}:${changeSetId}:${componentId}`;
  const cached = cache.get(cacheKey);
  if (cached) {
    logger.debug(`Component cache hit for ID: {componentId}`, {
      componentId,
    });
    return cached;
  }

  // Cache miss - perform API call
  logger.debug(`Component cache miss for ID: {componentId}`, {
    componentId,
  });

  const componentResult = await componentsApi.getComponent({
    workspaceId,
    changeSetId,
    componentId,
  });

  // Cache the result
  cache.set(cacheKey, componentResult.data);
  logger.debug(`Cached component data for ID: {componentId}`, {
    componentId,
  });

  return componentResult.data;
}

/**
 * Get schema data for a given schema ID, with caching.
 * Checks cache first, fetches from API if not cached.
 *
 * @param cache - Map to store cached schema responses
 * @param schemasApi - Initialized SchemasApi instance
 * @param logger - Logger instance for debug/trace logging
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param schemaId - Schema ID to lookup
 * @returns The schema data response
 */
export async function cachedGetSchema(
  cache: Map<string, GetSchemaV1Response>,
  schemasApi: SchemasApi,
  logger: Logger,
  workspaceId: string,
  changeSetId: string,
  schemaId: string,
): Promise<GetSchemaV1Response> {
  // Check cache first
  const cached = cache.get(schemaId);
  if (cached) {
    logger.debug(`Schema cache hit for ID: {schemaId}`, {
      schemaId,
    });
    return cached;
  }

  // Cache miss - perform API call
  logger.debug(`Schema cache miss for ID: {schemaId}`, {
    schemaId,
  });

  const response = await schemasApi.getSchema({
    workspaceId,
    changeSetId,
    schemaId,
  });

  // Cache the full response
  cache.set(schemaId, response.data);
  logger.debug(`Cached schema: {schemaId} -> {name}`, {
    schemaId,
    name: response.data.name,
  });

  return response.data;
}

/**
 * Get schema ID for a given schema name, with caching.
 * Uses the findSchema API to efficiently look up a single schema by name.
 * Checks cache first by iterating over cached entries, fetches from API if not cached.
 *
 * @param cache - Map to store cached schema responses
 * @param schemasApi - Initialized SchemasApi instance
 * @param logger - Logger instance for debug/trace logging
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param schemaName - Schema name to lookup (e.g., "AWS EC2 Instance")
 * @returns The schema ID
 * @throws Error if schema is not found or API call fails
 */
export async function cachedGetSchemaIdByName(
  cache: Map<string, GetSchemaV1Response>,
  schemasApi: SchemasApi,
  logger: Logger,
  workspaceId: string,
  changeSetId: string,
  schemaName: string,
): Promise<string> {
  // Check if we already have this schema cached by name
  for (const [schemaId, schemaData] of cache.entries()) {
    if (schemaData.name === schemaName) {
      logger.debug(
        `Schema cache hit for name: {schemaName} -> {schemaId}`,
        {
          schemaName,
          schemaId,
        },
      );
      return schemaId;
    }
  }

  // Cache miss - fetch from API using findSchema
  logger.debug(`Schema name cache miss for: {schemaName}`, { schemaName });

  const response = await schemasApi.findSchema({
    workspaceId,
    changeSetId,
    schema: schemaName,
  });

  const schemaId = response.data.schemaId;

  // Cache the result (convert FindSchemaV1Response to GetSchemaV1Response format)
  // We have limited data from findSchema, but cache what we have
  cache.set(schemaId, {
    name: response.data.schemaName,
    // FindSchemaV1Response doesn't include all fields, but we can cache the basics
  } as GetSchemaV1Response);

  logger.debug(`Found and cached schema: {schemaName} -> {schemaId}`, {
    schemaName,
    schemaId,
  });

  return schemaId;
}
