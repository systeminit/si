import type { GlobalOptions } from "../cli.ts";

export interface SecretCreateOptions extends GlobalOptions {
  secretType: string;
  name?: string;
  description?: string;
  changeSet?: string;
  useLocalProfile?: boolean;
  interactive?: boolean;
  dryRun?: boolean;
  fields?: Record<string, string>;
}

export interface SecretDefinitionV1 {
  formData: Array<SecretFormDataV1>;
  secretDefinition: string;
}

export interface SecretFormDataV1 {
  kind: string; // "string", "password", etc.
  name: string; // Field name
}

export type SecretFieldValues = Record<string, string>;
