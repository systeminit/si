/**
 * Component update command - Update component from file
 *
 * This module provides the command handler for updating components
 * from JSON/YAML configuration files.
 *
 * @module
 */

import type { Context } from "../../context.ts";
import {
  componentUpdate,
  type ComponentUpdateOptions,
} from "../../component/update.ts";

/**
 * Executes the component update command.
 *
 * @param ctx - The CLI context
 * @param inputFile - Path to input file (JSON or YAML)
 * @param options - Update command options
 */
export async function callComponentUpdate(
  _ctx: Context,
  inputFile: string,
  options: ComponentUpdateOptions,
) {
  await componentUpdate(inputFile, options);
}
