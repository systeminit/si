import type { SecretFieldValues, SecretFormDataV1 } from "./types.ts";

/**
 * Discover credentials from environment variables
 *
 * This is a completely generic discoverer that works with ANY secret type by:
 * 1. Looking at the secret's field names
 * 2. Trying multiple case/naming conventions for each field
 * 3. Checking common environment variable patterns
 * 4. Returning any discovered values
 *
 * @param formData - The secret definition's form fields
 * @returns Discovered field values, or null if nothing found
 */
export function discoverCredentials(
  formData: SecretFormDataV1[],
): SecretFieldValues | null {
  const discoveredValues: SecretFieldValues = {};

  // For each field in the secret schema, try to find a matching environment variable
  for (const field of formData) {
    const result = findEnvironmentVariable(field.name);
    if (result) {
      discoveredValues[field.name] = result.value;
    }
  }

  // Return null if nothing was discovered
  return Object.keys(discoveredValues).length > 0 ? discoveredValues : null;
}

/**
 * Find an environment variable value for a given field name
 * Tries multiple naming conventions to maximize discovery
 */
function findEnvironmentVariable(
  fieldName: string,
): { value: string; source: string } | null {
  // Generate all possible environment variable name variants
  const variants = generateEnvVarVariants(fieldName);

  // Try each exact variant first
  for (const variant of variants) {
    // deno-lint-ignore si-rules/no-deno-env-get -- Required for credential discovery
    const value = Deno.env.get(variant);
    if (value && value !== "undefined" && value.trim() !== "") {
      return { value, source: variant };
    }
  }

  // If no exact match, try pattern matching
  return findEnvironmentVariableByPattern(fieldName);
}

/**
 * Pattern match environment variables by looking for any variable that ends
 * with the field name pattern. This is completely generic and works with any
 * provider prefix (AWS_, AZURE_, ARM_, CUSTOM_PROVIDER_, etc.)
 */
function findEnvironmentVariableByPattern(
  fieldName: string,
): { value: string; source: string } | null {
  const envVars = Deno.env.toObject();

  // Convert field name to snake_case for matching
  const fieldNameSnake = fieldName
    .replace(/([A-Z])/g, "_$1")
    .replace(/^_/, "")
    .toUpperCase();

  for (const [envKey, envValue] of Object.entries(envVars)) {
    // Skip empty or undefined values
    if (!envValue || envValue === "undefined" || envValue.trim() === "") {
      continue;
    }

    // Skip very short env var names (like _, PATH, etc.)
    if (envKey.length < 3) {
      continue;
    }

    const envKeyUpper = envKey.toUpperCase();

    // Check if env var ends with the field name in snake_case
    // This matches:
    // - AWS_ACCESS_KEY_ID → AccessKeyId
    // - AZURE_CLIENT_SECRET → ClientSecret
    // - CUSTOM_PROVIDER_TOKEN → Token
    // - MY_COMPANY_API_KEY → ApiKey
    if (
      envKeyUpper.endsWith("_" + fieldNameSnake) ||
      envKeyUpper === fieldNameSnake
    ) {
      return { value: envValue, source: envKey };
    }
  }

  return null;
}

/**
 * Generate all possible environment variable name variants for a field name
 *
 * Examples:
 * - "AccessKeyId" -> ["ACCESS_KEY_ID", "ACCESSKEYID", "accessKeyId", "access_key_id", etc.]
 * - "ClientSecret" -> ["CLIENT_SECRET", "CLIENTSECRET", "clientSecret", "client_secret", etc.]
 * - "HetznerApiToken" -> ["HETZNER_API_TOKEN", "HETZNERAPITOKEN", "hetznerApiToken", etc.]
 */
function generateEnvVarVariants(fieldName: string): string[] {
  const variants: string[] = [];

  // 1. Exact field name (as-is)
  variants.push(fieldName);

  // 2. SCREAMING_SNAKE_CASE (most common for env vars)
  // Convert camelCase/PascalCase to snake_case, then uppercase
  const snakeCase = fieldName
    .replace(/([A-Z])/g, "_$1") // Add underscore before capitals
    .replace(/^_/, "") // Remove leading underscore
    .toLowerCase();

  variants.push(snakeCase.toUpperCase()); // ACCESS_KEY_ID
  variants.push(snakeCase); // access_key_id

  // 3. ALL UPPERCASE (no separators)
  variants.push(fieldName.toUpperCase()); // ACCESSKEYID

  // 4. All lowercase (no separators)
  variants.push(fieldName.toLowerCase()); // accesskeyid

  // 5. camelCase (first letter lowercase)
  if (fieldName.length > 0) {
    const camelCase = fieldName[0].toLowerCase() + fieldName.slice(1);
    variants.push(camelCase); // accessKeyId
  }

  // 6. PascalCase (first letter uppercase)
  if (fieldName.length > 0) {
    const pascalCase = fieldName[0].toUpperCase() + fieldName.slice(1);
    variants.push(pascalCase); // AccessKeyId
  }

  // Remove duplicates and return
  return [...new Set(variants)];
}
