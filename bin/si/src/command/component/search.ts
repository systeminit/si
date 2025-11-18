/**
 * Component search command - Search for components
 *
 * This module provides the command handler for searching components
 * in System Initiative workspaces.
 *
 * @module
 */

import type { Context } from "../../context.ts";
import {
  componentSearch,
  type ComponentSearchOptions,
} from "../../component/search.ts";

/**
 * Executes the component search command.
 *
 * @param ctx - The CLI context
 * @param query - Search query string
 * @param options - Search command options
 */
export async function callComponentSearch(
  _ctx: Context,
  query: string,
  options: ComponentSearchOptions,
) {
  await componentSearch(query, options);
}
