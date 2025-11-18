/**
 * Component delete command - Delete components
 *
 * This module provides the command handler for deleting components
 * from System Initiative workspaces.
 *
 * @module
 */

import type { Context } from "../../context.ts";
import {
  type ComponentDeleteOptions,
  deleteComponent,
} from "../../component/delete.ts";

/**
 * Executes the component delete command.
 *
 * @param ctx - The CLI context
 * @param component - Component name or ID
 * @param options - Delete command options
 */
export async function callComponentDelete(
  _ctx: Context,
  component: string,
  options: ComponentDeleteOptions,
) {
  await deleteComponent(component, options);
}
