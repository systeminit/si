/**
 * Project Module - SI Conduit Project Structure and Path Management
 *
 * This module provides utilities for working with SI Conduit project
 * structures, including path generation for schemas, actions, code generators,
 * management functions, qualifications, and their metadata files.
 *
 * ## Overview
 *
 * The Project module is the core utility for working with System Initiative
 * Conduit project file structures. It provides:
 *
 * - **Type-safe path generation** for all project resources
 * - **Consistent naming conventions** across the project structure
 * - **File system operations** through specialized path classes
 * - **Schema discovery** and enumeration
 *
 * ## Project Structure
 *
 * SI Conduit projects follow this directory structure:
 * ```
 * project-root/
 * ├── .conduitroot                    # Project marker file
 * └── schemas/                        # All schemas directory
 *     └── <schema-name>/              # Individual schema directory
 *         ├── .format-version         # Schema format version
 *         ├── schema.ts               # Schema definition code
 *         ├── schema.metadata.json    # Schema metadata
 *         ├── actions/                # Action functions directory
 *         │   ├── create.ts
 *         │   ├── create.metadata.json
 *         │   ├── destroy.ts
 *         │   ├── destroy.metadata.json
 *         │   ├── refresh.ts
 *         │   ├── refresh.metadata.json
 *         │   ├── update.ts
 *         │   └── update.metadata.json
 *         ├── codeGenerators/         # Code generator functions
 *         │   ├── <codegen-name>.ts
 *         │   └── <codegen-name>.metadata.json
 *         ├── management/             # Management functions
 *         │   ├── <management-name>.ts
 *         │   └── <management-name>.metadata.json
 *         └── qualifications/         # Qualification functions
 *             ├── <qualification-name>.ts
 *             └── <qualification-name>.metadata.json
 * ```
 *
 * ## Core Concepts
 *
 * ### Schemas
 *
 * Schemas define the structure and behavior of assets in System Initiative.
 * Each schema has its own directory containing:
 * - **`.format-version`**: Tracks the schema structure version for
 *   compatibility
 * - **`schema.ts`**: TypeScript code defining the schema structure
 * - **`schema.metadata.json`**: Configuration, display names, and metadata
 *
 * ## Path Type Safety
 *
 * This module provides type-safe path handling through specialized classes:
 *
 * ### Absolute Paths
 * - **`AbsolutePath`**: Base class for absolute paths
 * - **`AbsoluteDirectoryPath`**: Directory with `exists()` and `mkdir()`
 *   operations
 * - **`AbsoluteFilePath`**: File with `readTextFile()` and `writeTextFile()`
 *   operations
 *
 * ### Relative Paths
 * - **`RelativePath`**: Base class for relative paths
 * - **`RelativeDirectoryPath`**: Type-safe relative directory path
 * - **`RelativeFilePath`**: Type-safe relative file path
 *
 * These classes provide type safety and prevent mixing relative and absolute
 * paths incorrectly.
 * @module
 */

import { isAbsolute, join as pathJoin, normalize } from "@std/path";

/**
 * The filename for the project root marker file.
 *
 * This constant defines the marker file that identifies a directory as a System
 * Initiative Conduit project root. The presence of this file indicates that the
 * directory is the root of a conduit project.
 *
 * @internal
 */
const ROOT_MARKER_FILE = ".conduitroot" as const;

/** Directory name for schemas within a project. */
const SCHEMA_DIR = "schemas" as const;

/** Directory name for actions within a schema. */
const ACTIONS_DIR = "actions" as const;

/** Directory name for code generators within a schema. */
const CODEGENS_DIR = "codeGenerators" as const;

/** Directory name for management functions within a schema. */
const MANAGEMENTS_DIR = "management" as const;

/** Directory name for authentication functions within a schema. */
const AUTH_DIR = "authentication" as const;

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
 * schemas and their associated functions.
 *
 * This class encapsulates project structure knowledge and provides methods for
 * generating type-safe paths to various project files and directories. It
 * supports all function types (actions, code generators, management functions,
 * and qualifications) and provides both relative and absolute path variants.
 *
 * ## Path Generation Patterns
 *
 * The Project class follows consistent naming patterns for path methods:
 * - `*BaseRelativePath()` / `*BasePath()`: Directory paths
 * - `*FuncCodeRelativePath()` / `*FuncCodePath()`: TypeScript source files (.ts)
 * - `*MetadataRelativePath()` / `*MetadataPath()`: Metadata JSON files
 */
export class Project {
  /**
   * Default action function names commonly used in schemas.
   */
  static readonly DEFAULT_ACTION_NAMES = DEFAULT_ACTION_FUNCTION_NAMES;

  /**
   * Default codegen function names commonly used in schemas.
   */
  static readonly DEFAULT_CODEGEN_NAMES = DEFAULT_CODEGEN_FUNCTION_NAMES;

  /**
   * Default management function names commonly used in schemas.
   */
  static readonly DEFAULT_MANAGEMENT_NAMES = DEFAULT_MANAGEMENT_FUNCTION_NAMES;

  /**
   * Default qualification function names commonly used in schemas.
   */
  static readonly DEFAULT_QUALIFICATION_NAMES =
    DEFAULT_QUALIFICATION_FUNCTION_NAMES;

  /**
   * Creates a new Project instance.
   *
   * @param rootPath - The absolute path to the project root directory
   */
  constructor(public readonly rootPath: string) {}

  /**
   * Converts any path (relative or absolute) to an absolute directory path.
   *
   * This static utility method normalizes paths and resolves relative paths
   * against the current working directory. It's primarily used internally but
   * can be useful for normalizing user-provided paths.
   *
   * @param path - A relative or absolute path string
   * @returns An AbsoluteDirectoryPath instance with the normalized absolute
   * path
   */
  static projectBasePath(path: string): AbsoluteDirectoryPath {
    let absPath;
    if (isAbsolute(path)) {
      absPath = normalize(path);
    } else {
      absPath = normalize(join(Deno.cwd(), path));
    }

    return new AbsoluteDirectoryPath(absPath);
  }

  /**
   * Returns the absolute path to the project marker file (`.conduitroot`).
   *
   * This static utility method constructs the path to the `.conduitroot` marker
   * file that identifies a directory as a conduit project root. It's primarily
   * used by the RootPath module for project discovery.
   *
   * @param rootPath - The project root directory path
   * @returns An AbsoluteFilePath to the `.conduitroot` marker file
   */
  static projectMarkerPath(rootPath: AbsoluteDirectoryPath): AbsoluteFilePath {
    return new AbsoluteFilePath(join(rootPath, ROOT_MARKER_FILE));
  }

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
   * The format version file (`.format-version`) tracks the schema structure
   * version.
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
   * The format version file (`.format-version`) tracks the schema structure
   * version.
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
   * Returns the relative path to a schema's management functions directory.
   *
   * @param schemaName - The name of the schema
   * @returns A RelativeDirectoryPath to the authentication directory
   */
  authBaseRelativePath(schemaName: string): RelativeDirectoryPath {
    return new RelativeDirectoryPath(
      join(this.schemaBaseRelativePath(schemaName), AUTH_DIR),
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
   * Returns the absolute path to a schema's auth functions directory.
   *
   * @param schemaName - The name of the schema
   * @returns An AbsoluteDirectoryPath to the authentication directory
   */
  authBasePath(schemaName: string): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(this.rootPath, this.authBaseRelativePath(schemaName)),
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
 *
 * This is the base class for all absolute path types, providing common
 * functionality and type safety. Use the specialized subclasses
 * `AbsoluteDirectoryPath` and `AbsoluteFilePath` for directory and file
 * operations respectively.
 *
 * ## Design
 *
 * The path classes provide compile-time type safety to prevent mixing relative
 * and absolute paths incorrectly. They also provide a clean API surface for
 * filesystem operations appropriate to each path type.
 *
 * @example Basic usage
 * ```ts
 * const path = new AbsolutePath("/home/user/project");
 * console.log(path.toString()); // "/home/user/project"
 * console.log(path.path);       // "/home/user/project"
 * ```
 */
export class AbsolutePath {
  /**
   * Creates a new AbsolutePath instance.
   *
   * Note: This constructor does not validate that the path is actually
   * absolute. The caller is responsible for ensuring the path is absolute.
   * Consider using `Project.projectBasePath()` which handles both relative and
   * absolute paths correctly.
   *
   * @param path - The absolute path string
   */
  constructor(public readonly path: string) {}

  /**
   * Returns the string representation of the absolute path.
   *
   * This method allows path objects to be easily converted to strings for use
   * in logging, display, or with APIs that expect string paths.
   *
   * @returns The absolute path as a string
   *
   * @example
   * ```ts
   * const path = new AbsolutePath("/home/user");
   * console.log(`Path is: ${path.toString()}`);
   * // Or use implicit string conversion:
   * console.log(`Path is: ${path}`);
   * ```
   */
  public toString(): string {
    return this.path;
  }

  /**
   * Returns the string tag for this object type.
   *
   * Used by `Object.prototype.toString()` to provide a descriptive string
   * representation of the object type.
   *
   * @internal
   */
  get [Symbol.toStringTag](): string {
    return "AbsolutePath";
  }
}

/**
 * Represents an absolute directory path with directory-specific operations.
 *
 * Extends `AbsolutePath` with methods for checking directory existence and
 * creating directories. This class is returned by all Project methods that
 * generate directory paths (e.g., `schemaBasePath()`, `actionBasePath()`).
 *
 * ## Key Features
 *
 * - **Existence checking**: Async and synchronous methods to check if directory
 *   exists
 * - **Directory creation**: Support for creating directories with options like
 *   `recursive`
 * - **Type safety**: Prevents accidentally using file operations on directory
 *   paths
 */
export class AbsoluteDirectoryPath extends AbsolutePath {
  /**
   * Checks asynchronously if the directory exists at this path.
   *
   * This method specifically checks for a directory (not a file). If a file
   * exists at this path, the method returns `false`.
   *
   * @returns A promise resolving to `true` if a directory exists at this path,
   * `false` otherwise
   *
   * @example
   * ```ts
   * const schemasDir = project.schemasBasePath();
   *
   * if (await schemasDir.exists()) {
   *   const schemas = await project.currentSchemaDirNames();
   *   console.log(`Found ${schemas.length} schemas`);
   * } else {
   *   console.log("No schemas directory found");
   * }
   * ```
   */
  public async exists(): Promise<boolean> {
    return await dirExists(this);
  }

  /**
   * Checks synchronously if the directory exists at this path.
   *
   * This is the synchronous version of `exists()`. Use this when you need
   * immediate results without async/await, but prefer the async version when
   * possible to avoid blocking.
   *
   * @returns `true` if a directory exists at this path, `false` otherwise
   *
   * @example
   * ```ts
   * const rootPath = Project.projectBasePath("/path/to/project");
   *
   * if (rootPath.existsSync()) {
   *   console.log("Project directory exists");
   * } else {
   *   throw new Error("Project directory not found");
   * }
   * ```
   */
  public existsSync(): boolean {
    return dirExistsSync(this);
  }

  /**
   * Creates the directory at this path.
   *
   * This method wraps Deno's `mkdir` with the path from this instance. Common
   * options include:
   * - `recursive: true` - Create parent directories if they don't exist
   * - `mode` - Set permissions (Unix-like systems only)
   *
   * @param options - Optional mkdir options
   * @throws {Deno.errors.AlreadyExists} If the directory already exists and
   * recursive is not true
   * @throws {Deno.errors.NotFound} If parent directory doesn't exist and
   * recursive is not true
   *
   * @example Creating a directory with parents
   * ```ts
   * const schemaDir = project.schemaBasePath("NewSchema");
   * await schemaDir.mkdir({ recursive: true });
   * console.log("Created schema directory");
   * ```
   *
   * @example Creating directories conditionally
   * ```ts
   * const actionsDir = project.actionBasePath("MySchema");
   *
   * if (!(await actionsDir.exists())) {
   *   await actionsDir.mkdir({ recursive: true });
   *   console.log("Created actions directory");
   * }
   * ```
   *
   * @example Creating with specific permissions
   * ```ts
   * const dir = project.schemaBasePath("MySchema");
   * await dir.mkdir({ recursive: true, mode: 0o755 });
   * ```
   */
  public async mkdir(options?: Deno.MkdirOptions) {
    await Deno.mkdir(this.path, options);
  }
}

/**
 * Represents an absolute file path with file-specific operations.
 *
 * Extends `AbsolutePath` with methods for reading and writing text files,
 * checking file existence, and creating files. This class is returned by all
 * Project methods that generate file paths (e.g., `schemaFuncCodePath()`,
 * `actionFuncMetadataPath()`).
 *
 * ## Key Features
 *
 * - **Existence checking**: Async and synchronous methods to check if file
 *   exists
 * - **File creation**: Create empty files
 * - **Text file I/O**: Read and write text files with encoding support
 * - **Type safety**: Prevents accidentally using directory operations on file
 *   paths
 */
export class AbsoluteFilePath extends AbsolutePath {
  /**
   * Checks asynchronously if the file exists at this path.
   *
   * This method specifically checks for a file (not a directory). If a
   * directory exists at this path, the method returns `false`.
   *
   * @returns A promise resolving to `true` if a file exists at this path,
   * `false` otherwise
   *
   * @example
   * ```ts
   * const schemaPath = project.schemaFuncCodePath("MySchema");
   *
   * if (await schemaPath.exists()) {
   *   console.log("Schema file found");
   *   const code = await schemaPath.readTextFile();
   * } else {
   *   console.log("Schema file not found");
   * }
   * ```
   */
  public async exists(): Promise<boolean> {
    return await fileExists(this);
  }

  /**
   * Checks synchronously if the file exists at this path.
   *
   * This is the synchronous version of `exists()`. Use this when you need
   * immediate results without async/await, but prefer the async version when
   * possible to avoid blocking.
   *
   * @returns `true` if a file exists at this path, `false` otherwise
   *
   * @example
   * ```ts
   * const markerPath = Project.projectMarkerPath(rootPath);
   *
   * if (markerPath.existsSync()) {
   *   console.log("Project marker found");
   * } else {
   *   console.log("Not a conduit project");
   * }
   * ```
   */
  public existsSync(): boolean {
    return fileExistsSync(this);
  }

  /**
   * Creates an empty file at this path.
   *
   * This method creates a new empty file and returns a file handle. The file
   * handle should be closed when you're done with it. If you just want to
   * create an empty file marker, consider using `writeTextFile("")` instead.
   *
   * @returns A promise resolving to a Deno.FsFile handle
   * @throws {Deno.errors.AlreadyExists} If the file already exists
   * @throws {Deno.errors.NotFound} If parent directory doesn't exist
   *
   * @example Creating a marker file
   * ```ts
   * const markerPath = Project.projectMarkerPath(rootPath);
   * const file = await markerPath.create();
   * file.close();
   * console.log("Created project marker");
   * ```
   *
   * @example Simpler marker creation
   * ```ts
   * // Alternatively, use writeTextFile for marker files
   * await markerPath.writeTextFile("");
   * ```
   */
  public async create() {
    return await Deno.create(this.path);
  }

  /**
   * Writes text content to the file at this path.
   *
   * This method writes a string to the file, creating it if it doesn't exist or
   * overwriting it if it does. The content is encoded as UTF-8 by default.
   *
   * @param data - The text content to write
   * @param options - Optional write options
   * @throws {Deno.errors.NotFound} If parent directory doesn't exist
   *
   * @example Writing action code
   * ```ts
   * const actionPath = project.actionFuncCodePath("MySchema", "create");
   * const code = `export async function create(input: any) {
   *   console.log("Creating resource", input);
   *   return { success: true };
   * }`;
   * await actionPath.writeTextFile(code);
   * ```
   *
   * @example Appending to a file
   * ```ts
   * const logPath = project.schemaFuncCodePath("MySchema");
   * await logPath.writeTextFile("// Additional comment\n", { append: true });
   * ```
   */
  public async writeTextFile(data: string, options?: Deno.WriteFileOptions) {
    await Deno.writeTextFile(this.path, data, options);
  }

  /**
   * Reads text content from the file at this path.
   *
   * This method reads the entire file content as a UTF-8 encoded string. For
   * large files, consider using streaming APIs instead.
   *
   * @param options - Optional read options (e.g., signal for cancellation)
   * @returns A promise resolving to the file's text content
   * @throws {Deno.errors.NotFound} If the file doesn't exist
   * @throws {Deno.errors.PermissionDenied} If read permission is denied
   *
   * @example Reading schema metadata
   * ```ts
   * const metadataPath = project.schemaMetadataPath("MySchema");
   *
   * try {
   *   const content = await metadataPath.readTextFile();
   *   const metadata = JSON.parse(content);
   *   console.log(`Schema: ${metadata.name} v${metadata.version}`);
   * } catch (error) {
   *   if (error instanceof Deno.errors.NotFound) {
   *     console.log("Metadata file not found");
   *   } else {
   *     throw error;
   *   }
   * }
   * ```
   */
  public async readTextFile(options?: Deno.ReadFileOptions): Promise<string> {
    return await Deno.readTextFile(this.path, options);
  }
}

/**
 * Represents a relative file system path.
 *
 * This is the base class for relative path types, providing common
 * functionality and type safety. Relative paths are useful for:
 * - Display purposes (showing user-friendly paths)
 * - Configuration files (portable paths)
 * - Logging and error messages
 *
 * For file operations, convert to absolute paths using Project methods.
 *
 * @example Basic usage
 * ```ts
 * const relPath = new RelativePath("schemas/MySchema");
 * console.log(relPath.toString()); // "schemas/MySchema"
 * console.log(relPath.path);       // "schemas/MySchema"
 * ```
 *
 * @example Getting relative paths from Project
 * ```ts
 * const project = new Project("/home/user/my-project");
 *
 * // Get relative path for display
 * const relPath = project.schemaBaseRelativePath("MySchema");
 * console.log(`Schema location: ${relPath}`); // "schemas/MySchema"
 *
 * // Get absolute path for operations
 * const absPath = project.schemaBasePath("MySchema");
 * if (await absPath.exists()) {
 *   console.log("Schema exists");
 * }
 * ```
 */
export class RelativePath {
  /**
   * Creates a new RelativePath instance.
   *
   * Note: This constructor does not validate that the path is actually
   * relative. The caller is responsible for ensuring the path is relative.
   *
   * @param path - The relative path string
   */
  constructor(public readonly path: string) {}

  /**
   * Returns the string representation of the relative path.
   *
   * This method allows path objects to be easily converted to strings for use
   * in logging, display, or configuration files.
   *
   * @returns The relative path as a string
   *
   * @example
   * ```ts
   * const relPath = project.actionBaseRelativePath("MySchema");
   * console.log(`Actions dir: ${relPath.toString()}`);
   * // Output: Actions dir: schemas/MySchema/actions
   * ```
   */
  public toString(): string {
    return this.path;
  }

  /**
   * Returns the string tag for this object type.
   *
   * Used by `Object.prototype.toString()` to provide a descriptive string
   * representation of the object type.
   *
   * @internal
   */
  get [Symbol.toStringTag](): string {
    return "RelativePath";
  }
}

/**
 * Represents a relative directory path.
 *
 * This is a specialized version of RelativePath for directories, providing type
 * safety when working with directory paths in the project structure. Returned
 * by Project methods like `schemaBaseRelativePath()`,
 * `actionBaseRelativePath()`, etc.
 *
 * Relative directory paths are useful for:
 * - Displaying paths to users
 * - Writing to configuration files
 * - Logging and debugging output
 */
export class RelativeDirectoryPath extends RelativePath {}

/**
 * Represents a relative file path.
 *
 * This is a specialized version of RelativePath for files, providing type
 * safety when working with file paths in the project structure. Returned by
 * Project methods like `schemaFuncCodeRelativePath()`,
 * `actionFuncMetadataRelativePath()`, etc.
 *
 * Relative file paths are useful for:
 * - Displaying file locations to users
 * - Writing to configuration files
 * - Build system configurations
 * - Logging and debugging output
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
 * Synchronously checks if a directory exists at the given absolute path.
 *
 * This is the synchronous version of `dirExists()`. It uses exceptions as part
 * of its flow control, which is the recommended approach by the Deno project
 * for checking file existence.
 *
 * This function is used internally by `AbsoluteDirectoryPath.existsSync()`.
 *
 * @param absPath - The absolute path to check
 * @returns `true` if a directory exists at the path, `false` otherwise
 *
 * @see {@link https://docs.deno.com/examples/checking_file_existence/}
 * @internal
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
 * Synchronously checks if a file exists at the given absolute path.
 *
 * This is the synchronous version of `fileExists()`. It uses exceptions as part
 * of its flow control, which is the recommended approach by the Deno project
 * for checking file existence.
 *
 * This function is used internally by `AbsoluteFilePath.existsSync()`.
 *
 * @param absPath - The absolute path to check
 * @returns `true` if a file exists at the path, `false` otherwise
 *
 * @see {@link https://docs.deno.com/examples/checking_file_existence/}
 * @internal
 */
function fileExistsSync(absPath: AbsolutePath): boolean {
  try {
    const stat = Deno.lstatSync(absPath.toString());
    return stat.isFile;
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return false;
    }
    throw err;
  }
}

/**
 * Union type for path join operations, accepting strings or RelativePath
 * objects.
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
