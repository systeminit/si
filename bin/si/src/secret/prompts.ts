import { Input, Secret } from "@cliffy/prompt";
import type { SecretFieldValues, SecretFormDataV1 } from "./types.ts";

/**
 * Prompt for secret name
 */
export async function promptForSecretName(
  defaultName?: string,
): Promise<string> {
  const name = await Input.prompt({
    message: "Secret name:",
    default: defaultName,
  });

  if (!name || name.trim().length === 0) {
    throw new Error("Secret name is required");
  }

  return name.trim();
}

/**
 * Prompt for secret description
 */
export async function promptForDescription(
  defaultDescription?: string,
): Promise<string> {
  const description = await Input.prompt({
    message: "Description (optional):",
    default: defaultDescription || "",
  });

  return description.trim();
}

/**
 * Prompt for a single field value
 */
export async function promptForFieldValue(
  fieldName: string,
  fieldKind: string,
  defaultValue?: string,
): Promise<string> {
  const message = `${fieldName}:`;

  if (fieldKind === "password") {
    const value = await Secret.prompt({
      message,
      default: defaultValue,
    });
    return value || "";
  }

  const value = await Input.prompt({
    message,
    default: defaultValue || "",
  });

  return value || "";
}

/**
 * Prompt for all required field values
 */
export async function promptForFields(
  formData: SecretFormDataV1[],
  discoveredValues?: SecretFieldValues,
): Promise<SecretFieldValues> {
  const values: SecretFieldValues = {};

  for (const field of formData) {
    const defaultValue = discoveredValues?.[field.name];
    const value = await promptForFieldValue(
      field.name,
      field.kind,
      defaultValue,
    );

    if (value) {
      values[field.name] = value;
    }
  }

  return values;
}

/**
 * Confirm before proceeding with an action
 */
export async function confirmAction(message: string): Promise<boolean> {
  // Import Confirm dynamically to avoid issues
  const { Confirm } = await import("@cliffy/prompt");

  return await Confirm.prompt({
    message,
    default: true,
  });
}
