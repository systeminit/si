/**
 * Template run command - Execute SI template files
 *
 * This module provides functionality to run System Initiative template files,
 * which define and manage infrastructure components declaratively.
 *
 * @module
 */

import type { Context } from "../../context.ts";
import { runTemplate, type TemplateContextOptions } from "../../template.ts";

/**
 * Executes a template file with the specified options.
 *
 * @param ctx - The CLI context
 * @param template - Path to the template file
 * @param options - Template execution options
 */
export async function callRunTemplate(
  _ctx: Context,
  template: string,
  options: TemplateContextOptions,
) {
  await runTemplate(template, options);
}
