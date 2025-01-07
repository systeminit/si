import { Debug } from "../debug.ts";
import { bundle } from "jsr:@deno/emit";

const debug = Debug("langJs:package");

type Result<T> = {
  success: boolean;
  error?: string;
  value?: T;
};

export type ImportResult = Result<unknown>;

const SUPPORTED_IMPORT_TYPES = ["node:", "http:", "https:"] as const;
type SupportedImportType = typeof SUPPORTED_IMPORT_TYPES[number];

export const makePackage = (executionId: string) => {
  function isSupportedImportType(
    pkg: string,
  ): pkg is `${SupportedImportType}${string}` {
    return SUPPORTED_IMPORT_TYPES.some((type) => pkg.startsWith(type));
  }

  async function importPackage(pkg: string): Promise<ImportResult> {
    if (!isSupportedImportType(pkg)) {
      return {
        success: false,
        error: `Unsupported import type. Must start with one of: ${
          SUPPORTED_IMPORT_TYPES.join(", ")
        }`,
      };
    }

    try {
      debug(`${executionId} importing ${pkg}`);
      if (pkg.startsWith("http")) {
        const url = new URL(pkg);
        const { code } = await bundle(url);
        const dataUrl = `data:application/javascript;base64,${btoa(code)}`;
        pkg = dataUrl;
      }

      const module = await import(pkg);
      return {
        success: true,
        value: module,
      };
    } catch (error: unknown) {
      const errorMessage = error instanceof Error
        ? error.message
        : String(error);
      return {
        success: false,
        error: errorMessage,
      };
    }
  }

  return { importPackage };
};
