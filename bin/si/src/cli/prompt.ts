/**
 * Utility functions for prompting users for input in CLI commands.
 *
 * This module provides interactive prompts with suggestions and validation
 * for common CLI inputs like schema names, action names, and codegen names.
 * All prompt functions support both direct value passing (when provided) and
 * interactive prompting (when undefined), making them ideal for CLI commands
 * that can accept arguments or prompt interactively.
 *
 * @example
 * ```ts
 * import * as prompt from "./cli/prompt.ts";
 * import Project from "./project.ts";
 *
 * const project = new Project("/path/to/project");
 *
 * // Interactive prompt if schemaName is undefined
 * const schema = await prompt.schemaName(undefined, project);
 *
 * // Direct pass-through if value is provided
 * const action = await prompt.actionName("myAction", project);
 * ```
 *
 * @module
 */

import { Input } from "@cliffy/prompt";
import type { InputOptions } from "@cliffy/prompt/input";
import { Project } from "../schema/project.ts";
import { isInteractive } from "../logger.ts";
import { ValidationError } from "@cliffy/command";

/** Minimum length requirement for all prompt inputs. */
const MIN_INPUT_LENGTH = 1 as const;

/**
 * Creates a common prompt configuration with suggestions.
 *
 * This internal helper standardizes prompt creation by setting consistent
 * minimum length requirements and providing autocomplete suggestions to users.
 * The suggestions array is spread into a new array to avoid mutation issues.
 *
 * @param message - The prompt message to display to the user
 * @param suggestions - Array of suggestion strings for autocomplete (typically
 * derived from existing project entities or default naming conventions)
 * @returns Input options configuration object compatible with Cliffy's Input
 * prompt
 *
 * @internal
 */
function createPromptOptions(
  message: string,
  suggestions: readonly string[],
): InputOptions {
  return {
    message,
    minLength: MIN_INPUT_LENGTH,
    suggestions: [...suggestions],
  };
}

/**
 * Generic prompt helper that handles the common pattern of prompting for a
 * name.
 *
 * This internal helper reduces code duplication across all prompt functions by
 * implementing the flexible input strategy: if a value is provided, return it
 * immediately; otherwise, present an interactive prompt with suggestions.
 *
 * @param value - Optional value from command arguments
 * @param promptMessage - The message to display in the prompt
 * @param getSuggestions - Function that returns suggestions (can be sync or
 * async)
 * @returns A promise resolving to the value (either provided or prompted)
 *
 * @internal
 */
async function promptForName(
  value: string | undefined,
  promptMessage: string,
  getSuggestions: () => readonly string[] | Promise<readonly string[]>,
): Promise<string> {
  if (value !== undefined) {
    return value;
  }

  if (!isInteractive()) {
    throw new ValidationError(`Missing required argument for ${promptMessage}`);
  }

  const suggestions = await Promise.resolve(getSuggestions());
  return await Input.prompt(createPromptOptions(promptMessage, suggestions));
}

/**
 * Prompts the user for a schema name if not provided.
 *
 * This function implements a flexible input strategy: if a schema name is
 * provided, it returns immediately. Otherwise, it presents an interactive
 * prompt with autocomplete suggestions based on existing schema directories
 * in the project.
 *
 * @param schemaName - Optional schema name from command arguments. If provided,
 *   this value is returned as-is without prompting the user
 * @param project - Project instance used to retrieve existing schema directory
 *   names for autocomplete suggestions
 * @returns A promise resolving to the schema name (either the provided value
 *   or the user's prompted input)
 *
 * @example
 * ```ts
 * // With argument - no prompt shown
 * const name1 = await schemaName("MySchema", project);
 * // Returns: "MySchema"
 *
 * // Without argument - interactive prompt with suggestions
 * const name2 = await schemaName(undefined, project);
 * // User sees: "Schema Name: _" with autocomplete from existing schemas
 * ```
 *
 * @see {@link Project.currentSchemaDirNames}
 */
export async function schemaNameFromDirNames(
  schemaName: string | undefined,
  project: Project,
): Promise<string> {
  return await promptForName(
    schemaName,
    "Schema Name",
    () => project.schemas.currentSchemaDirNames(),
  );
}

export async function schemaName(
  schemaName: string | undefined,
  _project: Project,
): Promise<string> {
  return await promptForName(schemaName, "Schema Name", () => []);
}

/**
 * Prompts the user for an action name if not provided.
 *
 * This function implements a flexible input strategy: if an action name is
 * provided, it returns immediately. Otherwise, it presents an interactive
 * prompt with autocomplete suggestions based on default action function
 * naming conventions.
 *
 * @param actionName - Optional action name from command arguments. If provided,
 *   this value is returned as-is without prompting the user
 * @param project - Project instance used to retrieve default action function
 *   names for autocomplete suggestions (e.g., "create", "update", "delete")
 * @returns A promise resolving to the action name (either the provided value
 *   or the user's prompted input)
 *
 * @example
 * ```ts
 * // With argument - no prompt shown
 * const name1 = await actionName("customAction", project);
 * // Returns: "customAction"
 *
 * // Without argument - interactive prompt with default suggestions
 * const name2 = await actionName(undefined, project);
 * // User sees: "Action Function Name: _" with defaults like "create", "update"
 * ```
 *
 * @see {@link Project.defaultActionFunctionNames}
 */
export async function actionName(
  actionName: string | undefined,
  _project: Project,
): Promise<string> {
  return await promptForName(
    actionName,
    "Action Function Name",
    () => Project.DEFAULT_ACTION_NAMES,
  );
}

/**
 * Prompts the user for a codegen name if not provided.
 *
 * This function implements a flexible input strategy: if a codegen name is
 * provided, it returns immediately. Otherwise, it presents an interactive
 * prompt with autocomplete suggestions based on default codegen function
 * naming conventions.
 *
 * @param codegenName - Optional codegen name from command arguments. If
 * provided, this value is returned as-is without prompting the user
 * @param project - Project instance used to retrieve default codegen function
 * names for autocomplete suggestions
 * @returns A promise resolving to the codegen name (either the provided value
 * or the user's prompted input)
 *
 * @example
 * ```ts
 * // With argument - no prompt shown
 * const name1 = await codegenName("generateConfig", project);
 * // Returns: "generateConfig"
 *
 * // Without argument - interactive prompt with default suggestions
 * const name2 = await codegenName(undefined, project);
 * // User sees: "Codegen Function Name: _" with default naming suggestions
 * ```
 *
 * @see {@link Project.defaultCodegenFunctionNames}
 */
export async function codegenName(
  codegenName: string | undefined,
  _project: Project,
): Promise<string> {
  return await promptForName(
    codegenName,
    "Codegen Function Name",
    () => Project.DEFAULT_CODEGEN_NAMES,
  );
}

/**
 * Prompts the user for a management name if not provided.
 *
 * This function implements a flexible input strategy: if a management name is
 * provided, it returns immediately. Otherwise, it presents an interactive
 * prompt with autocomplete suggestions based on default management function
 * naming conventions.
 *
 * @param managementName - Optional management name from command arguments. If
 * provided, this value is returned as-is without prompting the user
 * @param project - Project instance used to retrieve default management
 * function names for autocomplete suggestions
 * @returns A promise resolving to the management name (either the provided
 * value or the user's prompted input)
 *
 * @example
 * ```ts
 * // With argument - no prompt shown
 * const name1 = await managementName("cleanup", project);
 * // Returns: "cleanup"
 *
 * // Without argument - interactive prompt with default suggestions
 * const name2 = await managementName(undefined, project);
 * // User sees: "Management Function Name: _" with default naming suggestions
 * ```
 *
 * @see {@link Project.defaultManagementFunctionNames}
 */
export async function managementName(
  managementName: string | undefined,
  _project: Project,
): Promise<string> {
  return await promptForName(
    managementName,
    "Management Function Name",
    () => Project.DEFAULT_MANAGEMENT_NAMES,
  );
}

/**
 * Prompts the user for a qualification name if not provided.
 *
 * This function implements a flexible input strategy: if a qualification name
 * is provided, it returns immediately. Otherwise, it presents an interactive
 * prompt with autocomplete suggestions based on default qualification function
 * naming conventions.
 *
 * @param qualificationName - Optional qualification name from command
 * arguments. If provided, this value is returned as-is without prompting the
 * user
 * @param project - Project instance used to retrieve default qualification
 * function names for autocomplete suggestions
 * @returns A promise resolving to the qualification name (either the provided
 * value or the user's prompted input)
 *
 * @example
 * ```ts
 * // With argument - no prompt shown
 * const name1 = await qualificationName("isValid", project);
 * // Returns: "isValid"
 *
 * // Without argument - interactive prompt with default suggestions
 * const name2 = await qualificationName(undefined, project);
 * // User sees: "Qualification Function Name: _" with default naming suggestions
 * ```
 *
 * @see {@link Project.defaultQualificationFunctionNames}
 */
export async function qualificationName(
  qualificationName: string | undefined,
  _project: Project,
): Promise<string> {
  return await promptForName(
    qualificationName,
    "Qualification Function Name",
    () => Project.DEFAULT_QUALIFICATION_NAMES,
  );
}

/**
 * Prompts the user for an authentication name if not provided.
 *
 * This function implements a flexible input strategy: if an authentication name
 * is provided, it returns immediately. Otherwise, it presents an interactive
 * prompt for the authentication function name.
 *
 * @param authName - Optional authentication name from command arguments.
 *   If provided, this value is returned as-is without prompting the user
 * @param project - Project instance
 * @returns A promise resolving to the authentication name (either the provided
 *   value or the user's prompted input)
 *
 * @example
 * ```ts
 * // With argument - no prompt shown
 * const name1 = await authName("oauth", project);
 * // Returns: "oauth"
 *
 * // Without argument - interactive prompt
 * const name2 = await authName(undefined, project);
 * // User sees: "Authentication Function Name: _"
 * ```
 */
export async function authName(
  authName: string | undefined,
  _project: Project,
): Promise<string> {
  return await promptForName(
    authName,
    "Authentication Function Name",
    () => [],
  );
}
