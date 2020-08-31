const ENC: Record<string, any> = {
  "+": "-",
  "/": "_",
  "=": ".",
};

export function urlSafeBase64Encode(content: string): string {
  let base64 = btoa(content);
  let urlSafeBase64 = base64.replace(/[+/=]/g, m => ENC[m]);
  let trimmedUrlSafeBase64 = urlSafeBase64.replace(/[.=]{1,2}$/, "");
  return trimmedUrlSafeBase64;
}
