import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { reactive } from "vue";

// token logic pulled from authstore

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

readTokens();
