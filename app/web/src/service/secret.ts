import { listSecretKindFields } from "./secret/list_secret_kinds";
import { createSecret } from "./secret/create_secret";
import { listSecrets } from "./secret/list_secrets";
import { getPublicKeyRaw } from "./secret/get_public_key";

export const SecretService = {
  createSecret,
  listSecrets,
  getPublicKeyRaw,
  listSecretKindFields,
};
