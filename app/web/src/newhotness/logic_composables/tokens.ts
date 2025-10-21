import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { reactive } from "vue";
import jwtDecode from "jwt-decode";

// token logic pulled from authstore

type TokenData = {
  user_pk: string;
  workspace_pk: string;
  // isImpersonating?: boolean;
};

const AUTH_LOCAL_STORAGE_KEYS = {
  USER_TOKENS: "si-auth",
};
export const tokensByWorkspacePk = reactive<Record<string, string>>({});

export const readTokens = () => {
  try {
    const parsed = JSON.parse(
      storage.getItem(AUTH_LOCAL_STORAGE_KEYS.USER_TOKENS) || "{}",
    );
    Object.entries(parsed).forEach(([k, v]) => {
      tokensByWorkspacePk[k] = v;
    });
  } catch {
    throw new Error("Failed loading tokens");
  }
};

export const getUserPkFromToken = (token: string): string => {
  const { user_pk: userPk } = jwtDecode<TokenData>(token);
  return userPk;
};

readTokens();
