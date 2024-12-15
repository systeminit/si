import { Debug } from "../debug.ts";

const debug = Debug("langJs:package");

type Result<T> = {
  success: true;
  value: T;
} | {
  success: false;
  error: string;
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
  async function importPackage(specifier: string): Promise<ImportResult> {
    if (!specifier.startsWith('node:')) {
      return {
        success: false,
        error: 'Only node: protocol imports are currently supported'
      };
    }

    try {
      debug(`${executionId} importing ${specifier}`);
      const module = await import(specifier);
      return {
        success: true,
        value: module
      };
    } catch (error: any) {
      debug(`Import failed for ${specifier}: ${error.message}`);
      return {
        success: false,
        error: error.message
      };
    }
  }

  return { importPackage }
}
