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

// I kept deleting this and bringing it back when debugging API client usage, so let's keep it here
export function logAllFunctions(obj: unknown) {
  if (typeof obj !== "object" || obj === null) {
    console.log("Not an object:", obj);
    return;
  }

  const objWithIndex = obj as Record<string, unknown>;
  const allPrototypeProps = Object.getOwnPropertyNames(
    Object.getPrototypeOf(objWithIndex),
  );
  const prototypeFunctions = allPrototypeProps.filter((name) =>
    typeof objWithIndex[name] === "function"
  );
  console.log(prototypeFunctions);
}
