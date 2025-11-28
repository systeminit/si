/**
 * Subscription resolution for component update command.
 *
 * This module provides utilities to resolve component names and search queries
 * in subscription attributes before sending updates to the API.
 *
 * @module
 */

import type { Logger } from "@logtape/logtape";
import type { SearchApi } from "@systeminit/api-client";
import { isSubscription } from "../template/attribute_diff.ts";
import {
  resolveComponentReference,
  resolveSearchQuery,
  type SearchFunction,
} from "./subscription_utils.ts";

/**
 * Resolves all subscription references in attributes to component IDs.
 *
 * This function processes subscription attributes that may contain:
 * - Component names → resolved to IDs via search
 * - Component IDs (ULIDs) → passed through unchanged
 * - Search queries (in query field) → executed and resolved to first matching component ID
 *
 * The attributes object is modified in-place, replacing names/queries with resolved IDs.
 *
 * @param attributes - Attributes object that may contain subscriptions
 * @param searchApi - SearchApi instance for executing searches
 * @param workspaceId - Workspace ID
 * @param changeSetId - Change set ID
 * @param logger - Logger instance
 */
export async function resolveSubscriptionsInAttributes(
  attributes: Record<string, unknown>,
  searchApi: SearchApi,
  workspaceId: string,
  changeSetId: string,
  logger: Logger,
): Promise<void> {
  // Create a simple search function that calls the API directly (no caching)
  const searchFn: SearchFunction = async (
    workspaceId: string,
    changeSetId: string,
    query: string,
  ) => {
    const response = await searchApi.search({
      workspaceId,
      changeSetId,
      q: query,
    });
    return response.data;
  };

  // Process each attribute
  for (const [path, value] of Object.entries(attributes)) {
    if (!isSubscription(value)) {
      continue;
    }

    logger.debug("Resolving subscription at {path}", { path });

    const subscription = value as {
      $source: {
        component?: string;
        query?: string;
        path: string;
        func?: string;
      };
    };

    // Detect subscription format by checking which field is present
    if (subscription.$source.query) {
      // Search-based subscription - execute search query
      const componentId = await resolveSearchQuery(
        subscription.$source.query,
        workspaceId,
        changeSetId,
        searchFn,
        logger,
      );

      // Update subscription in-place with resolved ID
      subscription.$source.component = componentId;
      delete subscription.$source.query;
    } else if (subscription.$source.component) {
      // Direct component reference - resolve if it's a name
      const componentId = await resolveComponentReference(
        subscription.$source.component,
        workspaceId,
        changeSetId,
        searchFn,
        logger,
      );

      // Update subscription in-place with resolved ID (might be unchanged if already ULID)
      subscription.$source.component = componentId;
    } else {
      throw new Error(
        `Invalid subscription at ${path}: must have either 'component' or 'query' field in $source`,
      );
    }
  }
}
