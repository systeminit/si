/**
 * Project Module - SI Project Structure and Path Management
 *
 * This module provides utilities for working with SI project
 * structures, including path generation for schemas, actions, code generators,
 * management functions, qualifications, and their metadata files.
 *
 * ## Overview
 *
 * The Project module is the core utility for working with System Initiative
 * project file structures. It provides:
 *
 * - **Type-safe path generation** for all project resources
 * - **Consistent naming conventions** across the project structure
 * - **File system operations** through specialized path classes
 * - **Schema discovery** and enumeration
 *
 * ## Project Structure
 *
 * SI projects follow this directory structure:
 * ```
 * project-root/
 * ├── .siroot                         # Project marker file
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

import { isAbsolute, join as pathJoin, normalize, relative } from "@std/path";
import { getLogger } from "./logger.ts";

const logger = getLogger();

/**
 * The filename for the project root marker file.
 *
 * This constant defines the marker file that identifies a directory as a System
 * Initiative project root. The presence of this file indicates that the
 * directory is the root of an SI project.
 *
 * @internal
 */
const ROOT_MARKER_FILE = ".siroot" as const;

/** Directory name for schemas within a project. */
const SCHEMA_DIR = "schemas" as const;

const OVERLAY_DIR = "overlays" as const;

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

export enum FunctionKind {
  Action = "action",
  Auth = "auth",
  Codegen = "codegen",
  Management = "management",
  Qualification = "qualification",
}

class ProjectModuleWithFunctions {
  constructor(public readonly moduleRootPath: string) {}

  moduleBaseRelativePath() {
    return join();
  }

  moduleBasePath() {
    return new AbsoluteDirectoryPath(
      join(this.moduleRootPath, this.moduleBaseRelativePath()),
    );
  }

  schemaBaseRelativePath(schemaName: string) {
    return new RelativeDirectoryPath(
      join(this.moduleBaseRelativePath(), normalizeFsName(schemaName)),
    );
  }

  schemaBasePath(schemaName: string): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(this.moduleRootPath, this.schemaBaseRelativePath(schemaName)),
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
    const schemasPath = this.moduleBasePath();

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

  // Function Methods
  funcBaseRelativePath(schemaName: string, funcKind: FunctionKind) {
    let dir: string;
    switch (funcKind) {
      case FunctionKind.Action:
        dir = ACTIONS_DIR;
        break;
      case FunctionKind.Auth:
        dir = AUTH_DIR;
        break;
      case FunctionKind.Codegen:
        dir = CODEGENS_DIR;
        break;
      case FunctionKind.Management:
        dir = MANAGEMENTS_DIR;
        break;
      case FunctionKind.Qualification:
        dir = QUALIFICATIONS_DIR;
        break;
      default:
        throw new Error(
          `Invalid function kind: ${funcKind} on ${this.moduleRootPath}`,
        );
    }

    return new RelativeDirectoryPath(
      join(this.schemaBaseRelativePath(schemaName), dir),
    );
  }

  funcBasePath(
    schemaName: string,
    funcKind: FunctionKind,
  ): AbsoluteDirectoryPath {
    return new AbsoluteDirectoryPath(
      join(
        this.moduleRootPath,
        this.funcBaseRelativePath(schemaName, funcKind),
      ),
    );
  }

  funcCodeRelativePath(
    schemaName: string,
    funcName: string,
    funcKind: FunctionKind,
  ) {
    return new RelativeFilePath(
      join(
        this.funcBaseRelativePath(schemaName, funcKind),
        `${normalizeFsName(funcName)}.ts`,
      ),
    );
  }

  funcCodePath(schemaName: string, funcName: string, funcKind: FunctionKind) {
    return new AbsoluteFilePath(
      join(
        this.moduleRootPath,
        this.funcCodeRelativePath(schemaName, funcName, funcKind),
      ),
    );
  }

  funcMetadataRelativePath(
    schemaName: string,
    funcName: string,
    funcKind: FunctionKind,
  ): RelativeFilePath {
    return new RelativeFilePath(
      join(
        this.funcBaseRelativePath(schemaName, funcKind),
        `${normalizeFsName(funcName)}.metadata.json`,
      ),
    );
  }

  funcMetadataPath(
    schemaName: string,
    funcName: string,
    funcKind: FunctionKind,
  ) {
    return new AbsoluteFilePath(
      join(
        this.moduleRootPath,
        this.funcMetadataRelativePath(schemaName, funcName, funcKind),
      ),
    );
  }
}

class SchemasProjectModule extends ProjectModuleWithFunctions {
  constructor(public override readonly moduleRootPath: string) {
    super(moduleRootPath);
  }

  formatVersionRelativePath(schemaName: string) {
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
  formatVersionPath(schemaName: string) {
    return new AbsoluteFilePath(
      join(this.moduleRootPath, this.formatVersionRelativePath(schemaName)),
    );
  }

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
      join(this.moduleRootPath, this.schemaFuncCodeRelativePath(schemaName)),
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
      join(this.moduleRootPath, this.schemaMetadataRelativePath(schemaName)),
    );
  }
}

/**
 * Represents a SI project and provides path utilities for working with
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

  schemas: SchemasProjectModule;
  overlays: ProjectModuleWithFunctions;

  /**
   * Creates a new Project instance.
   *
   * @param rootPath - The absolute path to the project root directory
   */
  constructor(public readonly rootPath: string) {
    this.schemas = new SchemasProjectModule(
      join(rootPath, SCHEMA_DIR),
    );
    this.overlays = new ProjectModuleWithFunctions(
      join(rootPath, OVERLAY_DIR),
    );
  }

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
   * Returns the absolute path to the project marker file (`.siroot`).
   *
   * This static utility method constructs the path to the `.siroot` marker
   * file that identifies a directory as an SI project root. It's primarily
   * used by the RootPath module for project discovery.
   *
   * @param rootPath - The project root directory path
   * @returns An AbsoluteFilePath to the `.siroot` marker file
   */
  static projectMarkerPath(rootPath: AbsoluteDirectoryPath): AbsoluteFilePath {
    return new AbsoluteFilePath(join(rootPath, ROOT_MARKER_FILE));
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
   * Computes the relative path from a base directory or project root to this
   * directory.
   *
   * This method calculates the relative path needed to navigate from the base
   * location to this directory. It's useful for displaying user-friendly paths,
   * generating portable configuration, or computing relative imports.
   *
   * @param base - The base directory or Project instance to compute relative to
   * @returns A RelativeDirectoryPath from the base to this directory
   *
   * @example Computing relative path from project root
   * ```ts
   * const project = new Project("/home/user/my-project");
   * const schemaDir = project.schemaBasePath("MySchema");
   *
   * const relPath = schemaDir.relativeTo(project);
   * console.log(relPath.toString()); // "schemas/MySchema"
   * ```
   *
   * @example Computing relative path between directories
   * ```ts
   * const schemasDir = project.schemasBasePath();
   * const schemaDir = project.schemaBasePath("MySchema");
   *
   * const relPath = schemaDir.relativeTo(schemasDir);
   * console.log(relPath.toString()); // "MySchema"
   * ```
   */
  public relativeTo(
    base: AbsoluteDirectoryPath | Project,
  ): RelativeDirectoryPath {
    if (base instanceof Project) {
      return new RelativeDirectoryPath(relative(base.rootPath, this.path));
    } else {
      return new RelativeDirectoryPath(relative(base.path, this.path));
    }
  }

  /**
   * Computes the relative path from a base directory or project root to this
   * directory and returns it as a string.
   *
   * This is a convenience method that combines `relativeTo()` and `toString()`.
   * It's useful when you need the relative path as a plain string for logging,
   * display, or passing to APIs that expect string paths.
   *
   * @param base - The base directory or Project instance to compute relative to
   * @returns The relative path as a string
   *
   * @example
   * ```ts
   * const project = new Project("/home/user/my-project");
   * const actionsDir = project.actionBasePath("MySchema");
   *
   * const relPath = actionsDir.relativeToStr(project);
   * console.log(`Actions located at: ${relPath}`);
   * // Output: Actions located at: schemas/MySchema/actions
   * ```
   */
  public relativeToStr(base: AbsoluteDirectoryPath | Project): string {
    return this.relativeTo(base).path;
  }

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
   * Computes the relative path from a base directory or project root to this
   * file.
   *
   * This method calculates the relative path needed to navigate from the base
   * location to this file. It's useful for displaying user-friendly file paths,
   * generating portable configuration, or computing relative imports.
   *
   * @param base - The base directory or Project instance to compute relative to
   * @returns A RelativeFilePath from the base to this file
   *
   * @example Computing relative path from project root
   * ```ts
   * const project = new Project("/home/user/my-project");
   * const schemaFile = project.schemaFuncCodePath("MySchema");
   *
   * const relPath = schemaFile.relativeTo(project);
   * console.log(relPath.toString()); // "schemas/MySchema/schema.ts"
   * ```
   *
   * @example Computing relative path for display
   * ```ts
   * const metadataPath = project.schemaMetadataPath("MySchema");
   * const relPath = metadataPath.relativeTo(project);
   * console.log(`Loading metadata from ${relPath}`);
   * // Output: Loading metadata from schemas/MySchema/schema.metadata.json
   * ```
   */
  public relativeTo(base: AbsoluteDirectoryPath | Project): RelativeFilePath {
    if (base instanceof Project) {
      return new RelativeFilePath(relative(base.rootPath, this.path));
    } else {
      return new RelativeFilePath(relative(base.path, this.path));
    }
  }

  /**
   * Computes the relative path from a base directory or project root to this
   * file and returns it as a string.
   *
   * This is a convenience method that combines `relativeTo()` and `toString()`.
   * It's useful when you need the relative path as a plain string for logging,
   * display, or passing to APIs that expect string paths.
   *
   * @param base - The base directory or Project instance to compute relative to
   * @returns The relative path as a string
   *
   * @example
   * ```ts
   * const project = new Project("/home/user/my-project");
   * const actionFile = project.actionFuncCodePath("MySchema", "create");
   *
   * const relPath = actionFile.relativeToStr(project);
   * console.log(`Reading action from: ${relPath}`);
   * // Output: Reading action from: schemas/MySchema/actions/create.ts
   * ```
   */
  public relativeToStr(base: AbsoluteDirectoryPath | Project): string {
    return this.relativeTo(base).path;
  }

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
   *   console.log("Not a SI project");
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
   * Writes text content to the file at this path using an atomic operation.
   *
   * This method writes a string to the file, creating it if it doesn't exist or
   * overwriting it if it does. The content is encoded as UTF-8 by default.
   *
   * **Atomic Write Safety**: To ensure data integrity, this method uses a
   * write-then-rename strategy:
   * 1. Creates a temporary file with a random suffix in the same directory
   * 2. Writes the content to the temporary file
   * 3. Atomically renames the temporary file to the target filename
   *
   * This approach ensures that the target file is never partially written, even
   * if the process is interrupted. Other processes will either see the old
   * content or the new content, never a corrupted intermediate state.
   *
   * @param data - The text content to write
   * @param options - Optional write options (applied to the temporary file)
   * @throws {Deno.errors.NotFound} If parent directory doesn't exist
   * @throws {Deno.errors.PermissionDenied} If write permission is denied
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
   * @example Writing metadata safely
   * ```ts
   * const metadataPath = project.schemaMetadataPath("MySchema");
   * const metadata = JSON.stringify({ name: "MySchema", version: "1.0.0" }, null, 2);
   * // Atomic write ensures metadata is never corrupted
   * await metadataPath.writeTextFile(metadata);
   * ```
   */
  public async writeTextFile(data: string, options?: Deno.WriteFileOptions) {
    const tmpFile = `${this.path}.tmp-${randomId()}`;

    // Create and write tmp file in same directory as destination
    logger.trace("Writing to tmp file {tmpFile}", { tmpFile });
    await Deno.writeTextFile(tmpFile, data, options);
    // Atomically move tmp file to destination file
    logger.trace("Moving tmp file {tmpFile} to {dst}", {
      tmpFile,
      dst: this.path,
    });
    await Deno.rename(tmpFile, this.path);
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

  return pathJoin(...(pathStrings as [string, ...string[]]));
}

/**
 * Normalizes a name for safe use in filesystem paths.
 *
 * Replaces any character that would be URI-encoded with a hyphen, as well as
 * the tilde character. Only these characters are kept: letters (A-Z, a-z),
 * digits (0-9), hyphen (-), underscore (_), and period (.).
 *
 * @param name - The name to normalize
 * @returns A filesystem-safe version of the name
 *
 * @example
 * ```ts
 * normalizeFsName("My Schema: Version 1?"); // "My-Schema--Version-1-"
 * normalizeFsName("User/Admin"); // "User-Admin"
 * normalizeFsName("test@example.com"); // "test-example.com"
 * normalizeFsName("file~name"); // "file-name"
 * ```
 */
export function normalizeFsName(name: string): string {
  return name.replace(/[^A-Za-z0-9._-]/g, "-");
}

/**
 * Generates a random identifier string for temporary file naming.
 *
 * Creates a 6-character hexadecimal string by generating a random number and
 * converting it to base-16. This is used internally by `writeTextFile()` to
 * create unique temporary file names during atomic write operations.
 *
 * The generated IDs are not cryptographically secure and should not be used
 * for security-sensitive purposes. They are suitable for creating temporary
 * file names where collisions are unlikely but not catastrophic.
 *
 * @returns A 6-character hexadecimal string (e.g., "a3f2c1", "9b4e7d")
 *
 * @example
 * ```ts
 * const id1 = randomId(); // "a3f2c1"
 * const id2 = randomId(); // "7e9d4b"
 * const tmpFile = `config.json.tmp-${randomId()}`; // "config.json.tmp-c8a5f2"
 * ```
 *
 * @internal
 *
 * Implementation thanks to Deno standard library.
 * @see {@link https://github.com/denoland/std/blob/16a70e9dac98256e0bb5714a0b9887e654fcef40/fs/_utils.ts#L58-L66}
 */
function randomId(): string {
  const n = (Math.random() * 0xfffff * 1_000_000).toString(16);
  return "".concat(n.slice(0, 6));
}
