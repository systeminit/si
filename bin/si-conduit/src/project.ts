/**
 * Project Module - SI Conduit Project Structure and Path Management
 *
 * This module provides utilities for working with SI Conduit project
 * structures, including path generation for schemas, actions, code generators,
 * management functions, qualifications, and their metadata files.
 *
 * ## Project Structure
 *
 * SI Conduit projects follow this directory structure:
 * ```
 * project-root/
 * └── schemas/
 *     └── <schema-name>/
 *         ├── .format-version
 *         ├── schema.ts
 *         ├── schema.metadata.json
 *         ├── actions/
 *         │   ├── create.ts
 *         │   ├── create.metadata.json
 *         │   ├── destroy.ts
 *         │   ├── destroy.metadata.json
 *         │   ├── refresh.ts
 *         │   ├── refresh.metadata.json
 *         │   ├── update.ts
 *         │   └── update.metadata.json
 *         ├── codeGenerators/
 *         │   ├── <codegen-name>.ts
 *         │   └── <codegen-name>.metadata.json
 *         ├── management/
 *         │   ├── <management-name>.ts
 *         │   └── <management-name>.metadata.json
 *         └── qualifications/
 *             ├── <qualification-name>.ts
 *             └── <qualification-name>.metadata.json
 * ```
 *
 * ## Core Concepts
 *
 * ### Schemas
 * Schemas define the structure and behavior of assets in System Initiative.
 * Each schema has its own directory containing:
 * - `.format-version`: Tracks the schema structure version
 * - `schema.ts`: Schema definition code
 * - `schema.metadata.json`: Schema configuration and metadata
 *
 * ### Function Types
 *
 * **Actions**: Functions that perform operations (create, destroy, refresh, update)
 *
 * **Code Generators**: Functions that generate code based on schema properties
 *
 * **Management Functions**: Functions for managing schema lifecycle and state
 *
 * **Qualifications**: Functions that validate or qualify schema instances
 *
 * ## Path Type Safety
 *
 * This module provides type-safe path handling through specialized classes:
 * - `AbsolutePath` / `AbsoluteDirectoryPath` / `AbsoluteFilePath`: For absolute paths
 * - `RelativePath` / `RelativeDirectoryPath` / `RelativeFilePath`: For relative paths
 *
 * ## Usage Examples
 *
 * @example Basic path operations
 * ```ts
 * import { Project } from "./project.ts";
 *
 * const project = new Project("/path/to/project");
 *
 * // Get path to a schema
 * const schemaPath = project.schemaBasePath("MySchema");
 * console.log(schemaPath.toString()); // "/path/to/project/schemas/MySchema"
 *
 * // Get path to an action function
 * const actionPath = project.actionFuncCodePath("MySchema", "create");
 * console.log(actionPath.toString()); // "/path/to/project/schemas/MySchema/actions/create.ts"
 * ```
 *
 * @example Working with schema files
 * ```ts
 * // Read schema metadata
 * const metadataPath = project.schemaMetadataPath("MySchema");
 * if (await metadataPath.exists()) {
 *   const content = await metadataPath.readTextFile();
 *   const metadata = JSON.parse(content);
 * }
 *
 * // Create action directory
 * const actionsDir = project.actionBasePath("MySchema");
 * await actionsDir.mkdir({ recursive: true });
 * ```
 *
 * @example Listing schemas
 * ```ts
 * // List all schemas in the project
 * const schemas = await project.currentSchemaDirNames();
 * console.log(schemas); // ["MySchema", "AnotherSchema"]
 * ```
 *
 * @module
 */

import { join as pathJoin } from "@std/path";

/** Directory name for schemas within a project. */
const SCHEMA_DIR = "schemas" as const;

/** Directory name for actions within a schema. */
const ACTIONS_DIR = "actions" as const;

/** Directory name for code generators within a schema. */
const CODEGENS_DIR = "codeGenerators" as const;

/** Directory name for management functions within a schema. */
const MANAGEMENTS_DIR = "management" as const;

/** Directory name for qualifications within a schema. */
const QUALIFICATIONS_DIR = "qualifications" as const;

/** Default action function names available for schemas. */
const DEFAULT_ACTION_FUNCTION_NAMES = [
  "create",
  "destroy",
  "refresh",
  "update",
] as const;

/** Default codegen function names available for schemas. */
const DEFAULT_CODEGEN_FUNCTION_NAMES = ["sample"] as const;

/** Default management function names available for schemas. */
const DEFAULT_MANAGEMENT_FUNCTION_NAMES = ["sample"] as const;

/** Default qualification function names available for schemas. */
const DEFAULT_QUALIFICATION_FUNCTION_NAMES = ["sample"] as const;

/**
 * Represents a SI Conduit project and provides path utilities for working with
 * schemas and action functions.
 *
 * This class encapsulates project structure knowledge and provides methods for
 * generating paths to various project files and directories.
 */
export class Project {
  /** Default action function names commonly used in schemas. */
  static readonly DEFAULT_ACTION_NAMES = DEFAULT_ACTION_FUNCTION_NAMES;

  /** Default codegen function names commonly used in schemas. */
  static readonly DEFAULT_CODEGEN_NAMES = DEFAULT_CODEGEN_FUNCTION_NAMES;

  /** Default management function names commonly used in schemas. */
  static readonly DEFAULT_MANAGEMENT_NAMES = DEFAULT_MANAGEMENT_FUNCTION_NAMES;

  /** Default qualification function names commonly used in schemas. */
  static readonly DEFAULT_QUALIFICATION_NAMES =
    DEFAULT_QUALIFICATION_FUNCTION_NAMES;

  /**
   * Creates a new Project instance.
   *
   * @param rootPath - The absolute path to the project root directory
   */
  constructor(public readonly rootPath: string) {}

  /**
   * Returns the relative path to the base schemas directory.
   *
   * @returns A RelativeDirectoryPath to the schemas directory
   */
  schemasBaseRelativePath(): RelativeDirectoryPath {
    return new RelativeDirectoryPath(join(SCHEMA_DIR));
  }

  /**
   * Returns the absolute path to the base schemas directory.
   *
   * @returns An AbsoluteDirectoryPath to the schemas directory
   */
  schemasBasePath(): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(this.rootPath, this.schemasBaseRelativePath()),
    );
  }

  /**
   * Returns the relative path to a specific schema's directory.
   *
   * Schema names are normalized for filesystem compatibility.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeDirectoryPath to the schema directory
   */
  schemaBaseRelativePath(schemaName: string): RelativeDirectoryPath {
    return new RelativeDirectoryPath(
      join(this.schemasBaseRelativePath(), normalizeFsName(schemaName)),
    );
  }

  /**
   * Returns the absolute path to a specific schema's directory.
   *
   * Schema names are normalized for filesystem compatibility.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteDirectoryPath to the schema directory
   */
  schemaBasePath(schemaName: string): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(this.rootPath, this.schemaBaseRelativePath(schemaName)),
    );
  }

  /**
   * Returns the relative path to a schema's format version file.
   *
   * The format version file (`.format-version`) tracks the schema structure version.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeFilePath to the format version file
   */
  schemaFormatVersionRelativePath(schemaName: string): RelativeFilePath {
    return new RelativeFilePath(
      join(this.schemaBaseRelativePath(schemaName), ".format-version"),
    );
  }

  /**
   * Returns the absolute path to a schema's format version file.
   *
   * The format version file (`.format-version`) tracks the schema structure version.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteFilePath to the format version file
   */
  schemaFormatVersionPath(schemaName: string): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(this.rootPath, this.schemaFormatVersionRelativePath(schemaName)),
    );
  }

  /**
   * Returns the relative path to a schema's function code file.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeFilePath to the schema's TypeScript file (schema.ts)
   */
  schemaFuncCodeRelativePath(schemaName: string): RelativeFilePath {
    return new RelativeFilePath(
      join(this.schemaBaseRelativePath(schemaName), "schema.ts"),
    );
  }

  /**
   * Returns the absolute path to a schema's function code file.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteFilePath to the schema's TypeScript file (schema.ts)
   */
  schemaFuncCodePath(schemaName: string): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(this.rootPath, this.schemaFuncCodeRelativePath(schemaName)),
    );
  }

  /**
   * Returns the relative path to a schema's metadata file.
   *
   * The metadata file contains schema configuration and properties.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeFilePath to the schema's metadata JSON file
   */
  schemaMetadataRelativePath(schemaName: string): RelativeFilePath {
    return new RelativeFilePath(
      join(this.schemaBaseRelativePath(schemaName), "schema.metadata.json"),
    );
  }

  /**
   * Returns the absolute path to a schema's metadata file.
   *
   * The metadata file contains schema configuration and properties.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteFilePath to the schema's metadata JSON file
   */
  schemaMetadataPath(schemaName: string): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(this.rootPath, this.schemaMetadataRelativePath(schemaName)),
    );
  }

  /**
   * Returns the relative path to a schema's actions directory.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeDirectoryPath to the actions directory
   */
  actionBaseRelativePath(schemaName: string): RelativeDirectoryPath {
    return new RelativeDirectoryPath(
      join(this.schemaBaseRelativePath(schemaName), ACTIONS_DIR),
    );
  }

  /**
   * Returns the absolute path to a schema's actions directory.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteDirectoryPath to the actions directory
   */
  actionBasePath(schemaName: string): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(this.rootPath, this.actionBaseRelativePath(schemaName)),
    );
  }

  /**
   * Returns the relative path to an action function's code file.
   *
   * @param schemaName - The name of the schema
   * @param actionName - The name of the action function
   * @returns A RelativePath to the action function's TypeScript file
   */
  actionFuncCodeRelativePath(
    schemaName: string,
    actionName: string,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(this.actionBaseRelativePath(schemaName), `${actionName}.ts`),
    );
  }

  /**
   * Returns the absolute path to an action function's code file.
   *
   * @param schemaName - The name of the schema
   * @param actionName - The name of the action function
   * @returns An AbsolutePath to the action function's TypeScript file
   */
  actionFuncCodePath(schemaName: string, actionName: string): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(
        this.rootPath,
        this.actionFuncCodeRelativePath(schemaName, actionName),
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
  actionFuncMetadataRelativePath(
    schemaName: string,
    actionName: string,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(
        this.actionBaseRelativePath(schemaName),
        `${actionName}.metadata.json`,
      ),
    );
  }

  /**
   * Returns the absolute path to an action function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param actionName - The name of the action function
   * @returns An AbsolutePath to the action function's metadata JSON file
   */
  actionFuncMetadataPath(
    schemaName: string,
    actionName: string,
  ): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(
        this.rootPath,
        this.actionFuncMetadataRelativePath(schemaName, actionName),
      ),
    );
  }

  /**
   * Returns the relative path to a schema's code generators directory.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeDirectoryPath to the codeGenerators directory
   */
  codegenBaseRelativePath(schemaName: string): RelativeDirectoryPath {
    return new RelativeDirectoryPath(
      join(this.schemaBaseRelativePath(schemaName), CODEGENS_DIR),
    );
  }

  /**
   * Returns the absolute path to a schema's code generators directory.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteDirectoryPath to the codeGenerators directory
   */
  codegenBasePath(schemaName: string): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(this.rootPath, this.codegenBaseRelativePath(schemaName)),
    );
  }

  /**
   * Returns the relative path to a code generator function's code file.
   *
   * @param schemaName - The name of the schema
   * @param codegenName - The name of the code generator function
   * @returns A RelativeFilePath to the code generator's TypeScript file
   */
  codegenFuncCodeRelativePath(
    schemaName: string,
    codegenName: string,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(this.codegenBaseRelativePath(schemaName), `${codegenName}.ts`),
    );
  }

  /**
   * Returns the absolute path to a code generator function's code file.
   *
   * @param schemaName - The name of the schema
   * @param codegenName - The name of the code generator function
   * @returns An AbsoluteFilePath to the code generator's TypeScript file
   */
  codegenFuncCodePath(
    schemaName: string,
    codegenName: string,
  ): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(
        this.rootPath,
        this.codegenFuncCodeRelativePath(schemaName, codegenName),
      ),
    );
  }

  /**
   * Returns the relative path to a code generator function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param codegenName - The name of the code generator function
   * @returns A RelativeFilePath to the code generator's metadata JSON file
   */
  codegenFuncMetadataRelativePath(
    schemaName: string,
    codegenName: string,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(
        this.codegenBaseRelativePath(schemaName),
        `${codegenName}.metadata.json`,
      ),
    );
  }

  /**
   * Returns the absolute path to a code generator function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param codegenName - The name of the code generator function
   * @returns An AbsoluteFilePath to the code generator's metadata JSON file
   */
  codegenFuncMetadataPath(
    schemaName: string,
    codegenName: string,
  ): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(
        this.rootPath,
        this.codegenFuncMetadataRelativePath(schemaName, codegenName),
      ),
    );
  }

  /**
   * Returns the relative path to a schema's management functions directory.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeDirectoryPath to the management directory
   */
  managementBaseRelativePath(schemaName: string): RelativeDirectoryPath {
    return new RelativeDirectoryPath(
      join(this.schemaBaseRelativePath(schemaName), MANAGEMENTS_DIR),
    );
  }

  /**
   * Returns the absolute path to a schema's management functions directory.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteDirectoryPath to the management directory
   */
  managementBasePath(schemaName: string): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(this.rootPath, this.managementBaseRelativePath(schemaName)),
    );
  }

  /**
   * Returns the relative path to a management function's code file.
   *
   * @param schemaName - The name of the schema
   * @param managementName - The name of the management function
   * @returns A RelativeFilePath to the management function's TypeScript file
   */
  managementFuncCodeRelativePath(
    schemaName: string,
    managementName: string,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(this.managementBaseRelativePath(schemaName), `${managementName}.ts`),
    );
  }

  /**
   * Returns the absolute path to a management function's code file.
   *
   * @param schemaName - The name of the schema
   * @param managementName - The name of the management function
   * @returns An AbsoluteFilePath to the management function's TypeScript file
   */
  managementFuncCodePath(
    schemaName: string,
    managementName: string,
  ): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(
        this.rootPath,
        this.managementFuncCodeRelativePath(schemaName, managementName),
      ),
    );
  }

  /**
   * Returns the relative path to a management function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param managementName - The name of the management function
   * @returns A RelativeFilePath to the management function's metadata JSON file
   */
  managementFuncMetadataRelativePath(
    schemaName: string,
    managementName: string,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(
        this.managementBaseRelativePath(schemaName),
        `${managementName}.metadata.json`,
      ),
    );
  }

  /**
   * Returns the absolute path to a management function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param managementName - The name of the management function
   * @returns An AbsoluteFilePath to the management function's metadata JSON file
   */
  managementFuncMetadataPath(
    schemaName: string,
    managementName: string,
  ): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(
        this.rootPath,
        this.managementFuncMetadataRelativePath(schemaName, managementName),
      ),
    );
  }

  /**
   * Returns the relative path to a schema's qualification functions directory.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeDirectoryPath to the qualifications directory
   */
  qualificationBaseRelativePath(schemaName: string): RelativeDirectoryPath {
    return new RelativeDirectoryPath(
      join(this.schemaBaseRelativePath(schemaName), QUALIFICATIONS_DIR),
    );
  }

  /**
   * Returns the absolute path to a schema's qualification functions directory.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteDirectoryPath to the qualifications directory
   */
  qualificationBasePath(schemaName: string): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(this.rootPath, this.qualificationBaseRelativePath(schemaName)),
    );
  }

  /**
   * Returns the relative path to a qualification function's code file.
   *
   * @param schemaName - The name of the schema
   * @param qualificationName - The name of the qualification function
   * @returns A RelativeFilePath to the qualification function's TypeScript file
   */
  qualificationFuncCodeRelativePath(
    schemaName: string,
    qualificationName: string,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(
        this.qualificationBaseRelativePath(schemaName),
        `${qualificationName}.ts`,
      ),
    );
  }

  /**
   * Returns the absolute path to a qualification function's code file.
   *
   * @param schemaName - The name of the schema
   * @param qualificationName - The name of the qualification function
   * @returns An AbsoluteFilePath to the qualification function's TypeScript file
   */
  qualificationFuncCodePath(
    schemaName: string,
    qualificationName: string,
  ): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(
        this.rootPath,
        this.qualificationFuncCodeRelativePath(schemaName, qualificationName),
      ),
    );
  }

  /**
   * Returns the relative path to a qualification function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param qualificationName - The name of the qualification function
   * @returns A RelativeFilePath to the qualification function's metadata JSON file
   */
  qualificationFuncMetadataRelativePath(
    schemaName: string,
    qualificationName: string,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(
        this.qualificationBaseRelativePath(schemaName),
        `${qualificationName}.metadata.json`,
      ),
    );
  }

  /**
   * Returns the absolute path to a qualification function's metadata file.
   *
   * @param schemaName - The name of the schema
   * @param qualificationName - The name of the qualification function
   * @returns An AbsoluteFilePath to the qualification function's metadata JSON file
   */
  qualificationFuncMetadataPath(
    schemaName: string,
    qualificationName: string,
  ): AbsoluteFilePath {
    return new AbsoluteFilePath(
      join(
        this.rootPath,
        this.qualificationFuncMetadataRelativePath(
          schemaName,
          qualificationName,
        ),
      ),
    );
  }

  /**
   * Returns the names of all schema directories currently in the project.
   *
   * Scans the schemas directory and returns an array of directory names.
   * Returns an empty array if the schemas directory doesn't exist.
   *
   * @returns A promise resolving to an array of schema directory names
   *
   * @example
   * ```ts
   * const schemas = await project.currentSchemaDirNames();
   * console.log(schemas); // ["UserSchema", "ProductSchema"]
   * ```
   */
  async currentSchemaDirNames(): Promise<string[]> {
    const schemasPath = new AbsolutePath(join(this.rootPath, SCHEMA_DIR));

    if (!(await dirExists(schemasPath))) {
      return [];
    }

    const entries = [];
    for await (const dirEntry of Deno.readDir(schemasPath.path)) {
      if (dirEntry.isDirectory) {
        entries.push(dirEntry.name);
      }
    }

    return entries;
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
 * Represents an absolute directory path with directory-specific operations.
 *
 * Extends AbsolutePath with methods for checking directory existence and
 * creating directories.
 */
export class AbsoluteDirectoryPath extends AbsolutePath {
  /**
   * Checks if the directory exists at this path.
   *
   * @returns A promise resolving to true if a directory exists at this path
   */
  public async exists(): Promise<boolean> {
    return await dirExists(this);
  }

  /**
   * Creates the directory at this path.
   *
   * @param options - Optional mkdir options (e.g., recursive, mode)
   * @throws {Deno.errors.AlreadyExists} If the directory already exists
   */
  public async mkdir(options?: Deno.MkdirOptions) {
    await Deno.mkdir(this.path, options);
  }
}

/**
 * Represents an absolute file path with file-specific operations.
 *
 * Extends AbsolutePath with methods for reading and writing text files,
 * and checking file existence.
 */
export class AbsoluteFilePath extends AbsolutePath {
  /**
   * Checks if the file exists at this path.
   *
   * @returns A promise resolving to true if a file exists at this path
   */
  public async exists(): Promise<boolean> {
    return await fileExists(this);
  }

  /**
   * Writes text content to the file at this path.
   *
   * @param data - The text content to write
   * @param options - Optional write options (e.g., mode, append, create)
   */
  public async writeTextFile(data: string, options?: Deno.WriteFileOptions) {
    await Deno.writeTextFile(this.path, data, options);
  }

  /**
   * Reads text content from the file at this path.
   *
   * @param options - Optional read options (e.g., signal)
   * @returns A promise resolving to the file's text content
   * @throws {Deno.errors.NotFound} If the file doesn't exist
   */
  public async readTextFile(options?: Deno.ReadFileOptions): Promise<string> {
    return await Deno.readTextFile(this.path, options);
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
 * Represents a relative directory path.
 *
 * This is a specialized version of RelativePath for directories, providing
 * type safety when working with directory paths in the project structure.
 */
export class RelativeDirectoryPath extends RelativePath {}

/**
 * Represents a relative file path.
 *
 * This is a specialized version of RelativePath for files, providing
 * type safety when working with file paths in the project structure.
 */
export class RelativeFilePath extends RelativePath {}

/**
 * Checks if a directory exists at the given absolute path.
 *
 * This implementation uses exceptions as part of its flow control, which is
 * the recommended approach by the Deno project for checking file existence.
 *
 * @param absPath - The absolute path to check
 * @returns A promise resolving to true if a directory exists at the path
 *
 * @see {@link https://docs.deno.com/examples/checking_file_existence/}
 * @internal
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
 * Checks if a file exists at the given absolute path.
 *
 * This implementation uses exceptions as part of its flow control, which is
 * the recommended approach by the Deno project for checking file existence.
 *
 * @param absPath - The absolute path to check
 * @returns A promise resolving to true if a file exists at the path
 *
 * @see {@link https://docs.deno.com/examples/checking_file_existence/}
 * @internal
 */
async function fileExists(absPath: AbsolutePath): Promise<boolean> {
  try {
    const stat = await Deno.lstat(absPath.toString());
    return stat.isFile;
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return false;
    }
    throw err;
  }
}

/**
 * Union type for path join operations, accepting strings or RelativePath objects.
 * @internal
 */
type StringOrRelativePath = string | RelativePath;

/**
 * Joins path segments into a single path string.
 *
 * Accepts both string path segments and RelativePath objects, automatically
 * extracting the path string from RelativePath instances.
 *
 * @param paths - Path segments to join (strings or RelativePath objects)
 * @returns The joined path string
 * @internal
 */
function join(...paths: StringOrRelativePath[]): string {
  const pathStrings = paths.map((element) => {
    if (typeof element === "string") {
      return element;
    } else {
      return element.path;
    }
  });

  return pathJoin(...pathStrings);
}

/**
 * Normalizes a name for safe use in filesystem paths.
 *
 * Replaces characters that are invalid in filesystem paths with hyphens.
 * The following characters are replaced: `\ / : * ? " < > |`
 *
 * @param name - The name to normalize
 * @returns A filesystem-safe version of the name
 *
 * @example
 * ```ts
 * normalizeFsName("My Schema: Version 1?"); // "My Schema- Version 1-"
 * normalizeFsName("User/Admin"); // "User-Admin"
 * ```
 */
export function normalizeFsName(name: string): string {
  return name.replace(/[\\/:*?"<>|]/g, "-");
}
