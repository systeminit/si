import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { reactive } from "vue";
import jwtDecode from "jwt-decode";

// token logic pulled from authstore

// V1 token format (legacy)
type TokenDataV1 = {
  user_pk: string;
  workspace_pk: string;
};

// V2 token format (new secure tokens)
type TokenDataV2 = {
  version: "2";
  userId: string;
  workspaceId: string;
  role: string;
  jti: string;
};

type TokenData = TokenDataV1 | TokenDataV2;

const AUTH_LOCAL_STORAGE_KEYS = {
  USER_TOKENS: "si-auth",
};
export const tokensByWorkspacePk = reactive<Record<string, string>>({});

export const readTokens = () => {
  try {
    const parsed = JSON.parse(storage.getItem(AUTH_LOCAL_STORAGE_KEYS.USER_TOKENS) || "{}");
    Object.entries(parsed).forEach(([k, v]) => {
      tokensByWorkspacePk[k] = v;
    });
  } catch {
    throw new Error("Failed loading tokens");
  }
};

export const getUserPkFromToken = (token: string): string => {
  const decoded = jwtDecode<TokenData>(token);

  // Check if V2 token format
  if ("version" in decoded && decoded.version === "2") {
    return decoded.userId;
  }

  // V1 token format
  const v1Token = decoded as TokenDataV1;
  return v1Token.user_pk;
};

readTokens();
