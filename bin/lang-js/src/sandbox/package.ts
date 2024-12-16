import { Debug } from "../debug.ts";

const debug = Debug("langJs:package");

type Result<T> = {
  success: boolean;
  error?: string;
  value?: T;
};

export type ImportResult = Result<unknown>;

export const makePackage = (executionId: string) => {
  /**
   * Imports a node package
   *
   * @example
   *  const crypto = await package.importPackage("node:crypto");
   *  const randomBytes = crypto.randomBytes(16).toString('hex');
   */
  async function importPackage(pkg: string): Promise<ImportResult> {
    if (!pkg.startsWith("node:")) {
      return {
        success: false,
        error: "Only node: protocol imports are currently supported",
      };
    }

    try {
      debug(`${executionId} importing ${pkg}`);
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
