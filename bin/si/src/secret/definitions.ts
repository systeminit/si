import { SecretsApi } from "@systeminit/api-client";
import type { Context } from "../context.ts";
import type { SecretDefinitionV1 } from "./types.ts";

export async function getSecretDefinitions(
  secretsApi: SecretsApi,
  workspaceId: string,
  changeSetId: string,
): Promise<SecretDefinitionV1[]> {
  const response = await secretsApi.getSecrets({
    workspaceId,
    changeSetId,
  });

  // Extract definitions from the grouped response
  const definitions: SecretDefinitionV1[] = [];

  for (const [_key, value] of Object.entries(response.data)) {
    if (value.definition) {
      definitions.push(value.definition);
    }
  }

  return definitions;
}

/**
 * Calculate string similarity using Levenshtein distance
 */
function levenshteinDistance(str1: string, str2: string): number {
  const matrix: number[][] = [];

  for (let i = 0; i <= str2.length; i++) {
    matrix[i] = [i];
  }

  for (let j = 0; j <= str1.length; j++) {
    matrix[0][j] = j;
  }

  for (let i = 1; i <= str2.length; i++) {
    for (let j = 1; j <= str1.length; j++) {
      if (str2.charAt(i - 1) === str1.charAt(j - 1)) {
        matrix[i][j] = matrix[i - 1][j - 1];
      } else {
        matrix[i][j] = Math.min(
          matrix[i - 1][j - 1] + 1,
          matrix[i][j - 1] + 1,
          matrix[i - 1][j] + 1,
        );
      }
    }
  }

  return matrix[str2.length][str1.length];
}

/**
 * Calculate similarity score between two strings (0-1, higher is more similar)
 */
function similarityScore(str1: string, str2: string): number {
  const longer = str1.length > str2.length ? str1 : str2;
  const shorter = str1.length > str2.length ? str2 : str1;

  if (longer.length === 0) {
    return 1.0;
  }

  const distance = levenshteinDistance(longer, shorter);
  return (longer.length - distance) / longer.length;
}

/**
 * Match a user-provided secret type string to available definitions
 */
export function matchSecretType(
  userInput: string,
  definitions: SecretDefinitionV1[],
  ctx: Context,
): SecretDefinitionV1 | null {
  const normalizedInput = userInput.toLowerCase().trim();

  // Try exact match first (case-insensitive)
  const exactMatch = definitions.find(
    (def) => def.secretDefinition.toLowerCase() === normalizedInput,
  );

  if (exactMatch) {
    return exactMatch;
  }

  // Try fuzzy matching
  const matches = definitions
    .map((def) => ({
      definition: def,
      score: similarityScore(
        normalizedInput,
        def.secretDefinition.toLowerCase(),
      ),
    }))
    .filter((match) => match.score > 0.5) // Only consider reasonable matches
    .sort((a, b) => b.score - a.score);

  if (matches.length === 0) {
    return null;
  }

  // If we have a very strong match (> 0.8), use it
  if (matches[0].score > 0.8) {
    ctx.logger.info(
      `Using secret type "${
        matches[0].definition.secretDefinition
      }" (closest match to "${userInput}")`,
    );
    return matches[0].definition;
  }

  // Otherwise, we have ambiguous results
  return null;
}

/**
 * List all available secret types in a formatted way
 */
export function listAvailableSecretTypes(
  definitions: SecretDefinitionV1[],
): string {
  if (definitions.length === 0) {
    return "No secret types available";
  }

  const types = definitions
    .map((def) => def.secretDefinition)
    .sort()
    .map((type) => `  - ${type}`)
    .join("\n");

  return `Available secret types:\n${types}`;
}

/**
 * Suggest similar secret types when a match fails
 */
export function suggestSimilarSecretTypes(
  userInput: string,
  definitions: SecretDefinitionV1[],
  maxSuggestions: number = 3,
): string[] {
  const normalizedInput = userInput.toLowerCase().trim();

  const suggestions = definitions
    .map((def) => ({
      type: def.secretDefinition,
      score: similarityScore(
        normalizedInput,
        def.secretDefinition.toLowerCase(),
      ),
    }))
    .filter((match) => match.score > 0.3)
    .sort((a, b) => b.score - a.score)
    .slice(0, maxSuggestions)
    .map((match) => match.type);

  return suggestions;
}
