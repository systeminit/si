import { join } from "https://deno.land/std/path/mod.ts";

/** Directory name for schemas within a project. */
const SCHEMA_DIR = "schemas" as const;
/** Directory name for actions within a schema. */
const ACTION_DIR = "actions" as const;

/** Default action function names available for schemas. */
const DEFAULT_ACTION_FUNCTION_NAMES = [
  "create",
  "destroy",
  "refresh",
  "update",
] as const;

/**
 * Represents a si-conduit project and provides path utilities for working with
 * schemas and action functions.
 */
export class Project {
  /**
   * Creates a new Project instance.
   *
   * @param rootPath - The absolute path to the project root directory
   */
  constructor(public readonly rootPath: string) {}

  /**
   * Returns the relative path to an action function's code file.
   *
   * @param schemaName - The name of the schema
   * @param actionName - The name of the action function
   * @returns A RelativePath to the action function's TypeScript file
   */
  relativeActionFuncCodePath(
    schemaName: string,
    actionName: string,
  ): RelativePath {
    return new RelativePath(
      join(SCHEMA_DIR, schemaName, ACTION_DIR, `${actionName}.ts`),
    );
  }

  /**
   * Returns the absolute path to an action function's code file.
   *
   * @param schemaName - The name of the schema
   * @param actionName - The name of the action function
   * @returns An AbsolutePath to the action function's TypeScript file
   */
  actionFuncCodePath(schemaName: string, actionName: string): AbsolutePath {
    return new AbsolutePath(
      join(
        this.rootPath,
        this.relativeActionFuncCodePath(schemaName, actionName).path,
      ),
    );
  }

  /**
   * Returns the relative path to an action function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param actionName - The name of the action function
   * @returns A RelativePath to the action function's metadata JSON file
   */
  relativeActionFuncMetadataPath(
    schemaName: string,
    actionName: string,
  ): RelativePath {
    return new RelativePath(
      join(SCHEMA_DIR, schemaName, ACTION_DIR, `${actionName}.metadata.json`),
    );
  }

  /**
   * Returns the absolute path to an action function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param actionName - The name of the action function
   * @returns An AbsolutePath to the action function's metadata JSON file
   */
  actionFuncMetadataPath(schemaName: string, actionName: string): AbsolutePath {
    return new AbsolutePath(
      join(
        this.rootPath,
        this.relativeActionFuncMetadataPath(schemaName, actionName).path,
      ),
    );
  }

  /**
   * Returns the default action function names.
   *
   * @returns A readonly array of default action function names
   */
  defaultActionFunctionNames(): ReadonlyArray<string> {
    return DEFAULT_ACTION_FUNCTION_NAMES;
  }

  /**
   * Returns the names of all schema directories currently in the project.
   *
   * @returns An array of schema directory names
   */
  async currentSchemaDirNames(): Promise<string[]> {
    const results: string[] = [];

    const schemasPath = new AbsolutePath(join(this.rootPath, SCHEMA_DIR));
    if (!(await dirExists(schemasPath))) {
      return [];
    }

    for await (const dirEntry of Deno.readDir(schemasPath.path)) {
      if (dirEntry.isDirectory) {
        results.push(dirEntry.name);
      }
    }

    return results;
  }
}

/**
 * Represents an absolute file system path.
 */
export class AbsolutePath {
  /**
   * Creates a new AbsolutePath instance.
   *
   * @param path - The absolute path string
   */
  constructor(public readonly path: string) {}

  /**
   * Returns the string representation of the absolute path.
   *
   * @returns The absolute path as a string
   */
  public toString(): string {
    return this.path;
  }

  get [Symbol.toStringTag](): string {
    return "AbsolutePath";
  }
}

/**
 * Represents a relative file system path.
 */
export class RelativePath {
  /**
   * Creates a new RelativePath instance.
   *
   * @param path - The relative path string
   */
  constructor(public readonly path: string) {}

  /**
   * Returns the string representation of the relative path.
   *
   * @returns The relative path as a string
   */
  public toString(): string {
    return this.path;
  }

  get [Symbol.toStringTag](): string {
    return "RelativePath";
  }
}

/**
 * Checks if a directory exists at the given absolute path.
 *
 * NOTE: this implementation uses exceptions as part of its flow control. While
 * not ideal, this is recommended by the Deno project in this case.
 *
 * @param absPath - The absolute path to check
 * @returns Whether a directory exists at the path
 *
 * @see {@link https://docs.deno.com/examples/checking_file_existence/}
 */
async function dirExists(absPath: AbsolutePath): Promise<boolean> {
  try {
    const stat = await Deno.lstat(absPath.toString());
    return stat.isDirectory;
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return false;
    }
    throw err;
  }
}

/**
 * Synchronously checks if a directory exists at the given absolute path.
 *
 * NOTE: this implementation uses exceptions as part of its flow control. While
 * not ideal, this is recommended by the Deno project in this case.
 *
 * @param absPath - The absolute path to check
 * @returns Whether a directory exists at the path
 *
 * @see {@link https://docs.deno.com/examples/checking_file_existence/}
 */
function dirExistsSync(absPath: AbsolutePath): boolean {
  try {
    const stat = Deno.lstatSync(absPath.toString());
    return stat.isDirectory;
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return false;
    }
    throw err;
  }
}
