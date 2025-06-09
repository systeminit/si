import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory

// token logic pulled from authstore

const AUTH_LOCAL_STORAGE_KEYS = {
  USER_TOKENS: "si-auth",
};
export const tokensByWorkspacePk: Record<string, string> = {};
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
