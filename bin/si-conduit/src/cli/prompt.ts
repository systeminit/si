/**
 * Utility functions for prompting users for input in CLI commands.
 *
 * This module provides interactive prompts with suggestions and validation
 * for common CLI inputs like schema names, action names, and codegen names.
 *
 * @module
 */

import { Input } from "@cliffy/prompt";
import type { InputOptions } from "@cliffy/prompt/input";
import { Project } from "./../project.ts";

/** Default codegen function name suggestions. */
const DEFAULT_CODEGEN_NAMES = ["default"] as const;

/** Minimum length requirement for all prompt inputs. */
const MIN_INPUT_LENGTH = 1 as const;

/**
 * Creates a common prompt configuration with suggestions.
 *
 * @param message - The prompt message to display
 * @param suggestions - Array of suggestion strings
 * @returns Input options configuration
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
 * Prompts the user for a schema name if not provided.
 *
 * @param schemaName - Optional schema name from command arguments
 * @param project - Project instance to get existing schema suggestions
 * @returns The schema name (provided or prompted)
 */
export async function schemaName(
  schemaName: string | undefined,
  project: Project,
): Promise<string> {
  return (
    schemaName ??
      (await Input.prompt(
        createPromptOptions(
          "Schema Name",
          await project.currentSchemaDirNames(),
        ),
      ))
  );
}

/**
 * Prompts the user for an action name if not provided.
 *
 * @param actionName - Optional action name from command arguments
 * @param project - Project instance to get default action suggestions
 * @returns The action name (provided or prompted)
 */
export async function actionName(
  actionName: string | undefined,
  project: Project,
): Promise<string> {
  return (
    actionName ??
      (await Input.prompt(
        createPromptOptions(
          "Action Function Name",
          project.defaultActionFunctionNames(),
        ),
      ))
  );
}

/**
 * Prompts the user for a codegen name if not provided.
 *
 * @param codegenName - Optional codegen name from command arguments
 * @returns The codegen name (provided or prompted)
 */
export async function codegenName(
  codegenName: string | undefined,
): Promise<string> {
  return (
    codegenName ??
      (await Input.prompt(
        createPromptOptions("Codegen Function Name", DEFAULT_CODEGEN_NAMES),
      ))
  );
}
