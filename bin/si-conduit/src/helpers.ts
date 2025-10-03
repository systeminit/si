import { AxiosError } from "axios";

export function unknownValueToErrorMessage(value: unknown): string {
  if (typeof value === "string") return value;

  if (value instanceof AxiosError && value.response?.data?.error?.message) {
    const status = value.response.status;
    const msg = value.response.data.error.message;
    return `HTTP ${status}: ${msg}`;
  }

  if (value instanceof Error) return value.message;

  return `Unknown Error: ${value}`;
}

export function makeStringSafeForFilename(str: string): string {
  return str.replace(/[\\/:*?"<>|]/g, "_");
}
