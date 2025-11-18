/**
 * Component get command - Fetch and display component data
 *
 * This module provides the command handler for retrieving component information
 * from System Initiative workspaces.
 *
 * @module
 */

import type { Context } from "../../context.ts";
import { type ComponentGetOptions, getComponent } from "../../component/get.ts";

/**
 * Executes the component get command.
 *
 * @param ctx - The CLI context
 * @param component - Component name or ID
 * @param options - Get command options
 */
export async function callComponentGet(
  _ctx: Context,
  component: string,
  options: ComponentGetOptions,
) {
  await getComponent(component, options);
}
