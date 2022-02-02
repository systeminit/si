const ENC: Record<string, string> = {
  "+": "-",
  "/": "_",
  "=": ".",
};

export function urlSafeBase64Encode(content: string): string {
  const base64 = btoa(content);
  const urlSafeBase64 = base64.replace(/[+/=]/g, (m) => ENC[m]);
  const trimmedUrlSafeBase64 = urlSafeBase64.replace(/[.=]{1,2}$/, "");
  return trimmedUrlSafeBase64;
}
