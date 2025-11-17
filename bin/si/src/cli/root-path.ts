/**
 * Project root path discovery and validation utilities.
 *
 * This module provides functionality to find and validate project root
 * directories by searching for a `.siroot` marker file. It includes both a
 * RootPath class for programmatic use and a RootPathType for use with Cliffy
 * CLI commands.
 *
 * ## Overview
 *
 * The module provides two main components:
 *
 * - **RootPath**: A class for programmatically finding and validating project
 *   roots
 * - **RootPathType**: A Cliffy Type for CLI argument parsing and validation
 *
 * ### Project Root Marker
 *
 * An SI project root is identified by the presence of a `.siroot` file
 * in the directory. This marker file indicates that the directory is the root
 * of a System Initiative project.
 *
 * ## Error Handling
 *
 * The module provides structured error handling:
 *
 * - **RootPathNotFoundError**: Returned by `from()`, contains the path that was
 *   checked
 * - **ValidationError**: Thrown by `find()` and `findFromCwd()`, includes
 *   helpful error messages
 *
 * @module
 */

import { type ArgumentValue, Type, ValidationError } from "@cliffy/command";
import { dirname } from "@std/path";
import { Project } from "../project.ts";

/** The marker file used to identify an SI project root directory. */
const ROOT_MARKER = ".siroot";

/**
 * Error thrown when a project root directory cannot be found or validated.
 *
 * This error extends Cliffy's ValidationError and is thrown when:
 * - A directory does not contain a `.siroot` marker file
 * - No `.siroot` file is found when searching upward through the directory
 *   tree
 */
export class RootPathNotFoundError extends ValidationError {
  /**
   * Creates a new RootPathNotFoundError.
   *
   * @param path - The path where the `.siroot` marker file was not found
   */
  constructor(public readonly path: string) {
    super(
      `No '${ROOT_MARKER}' file found in ${path}.\n\n  ` +
        "Use the 'project init' subcommand to initialize a project.",
    );
    this.name = "RootPathNotFoundError";
  }
}

/**
 * Represents a validated project root directory path.
 *
 * This class provides methods to validate and find project root directories,
 * either by directly validating a given path or by searching upwards from a
 * starting directory for a `.siroot` marker file.
 */
export class RootPath {
  /**
   * Creates a new RootPath instance.
   *
   * @param path - The absolute path to the project root directory
   */
  constructor(public readonly path: string) {}

  /**
   * Finds the project root by searching upwards from the current working
   * directory.
   *
   * This is a convenience method that calls `find()` with the current working
   * directory as the starting point.
   *
   * @returns The RootPath instance for the project root
   * @throws {ValidationError} If no `.siroot` file is found in the current
   * directory or any parent directory up to the filesystem root
   */
  static findFromCwd(): RootPath {
    return this.find(Deno.cwd());
  }

  /**
   * Finds the project root by searching upwards from the given start directory
   * for a `.siroot` file.
   *
   * This method implements an upward directory traversal algorithm:
   * 1. Starts at the given directory (or current working directory if "." is
   *    provided)
   * 2. Checks if a `.siroot` file exists in the current directory
   * 3. If found, returns the directory containing the marker file
   * 4. If not found, moves to the parent directory and repeats
   * 5. Stops when reaching the filesystem root
   *
   * @param startDir - The directory to start searching from. Can be an absolute
   * path or "." for the current working directory
   * @returns A RootPath instance pointing to the directory containing the
   * `.siroot` marker file
   * @throws {ValidationError} If the start directory doesn't exist
   * @throws {ValidationError} If no `.siroot` file is found in the start
   * directory or any parent directory up to the filesystem root
   */
  static find(startDir: string): RootPath {
    const start = startDir === "." ? Deno.cwd() : startDir;

    if (!Project.projectBasePath(start).existsSync()) {
      throw new ValidationError(`directory not found: ${start}`);
    }
    let currentDir = start;

    while (true) {
      const candidateRootMarkerPath = Project.projectMarkerPath(
        Project.projectBasePath(currentDir),
      );

      if (candidateRootMarkerPath.existsSync()) {
        return new RootPath(dirname(candidateRootMarkerPath.toString()));
      }

      const parentDir = dirname(currentDir);

      // If we've reached the root of the filesystem, stop
      if (parentDir === currentDir) {
        throw new ValidationError(
          `No ${ROOT_MARKER} file found in ${start} or any parent directory`,
        );
      }

      currentDir = parentDir;
    }
  }

  /**
   * Validates and creates a RootPath from the given path.
   *
   * This method validates that the given path directly contains a
   * `.siroot` marker file, without searching parent directories. This is
   * useful when you want to verify that a specific directory is a project root,
   * not just find the nearest project root.
   *
   * **Key Difference from `find`:**
   * - `from()` only checks the exact path provided - no upward search
   * - `find()` searches upward through parent directories
   *
   * This method returns either a `RootPath` on success or a
   * `RootPathNotFoundError` on failure, rather than throwing an exception. This
   * makes it suitable for use cases where you want to handle the error
   * gracefully or provide custom error handling.
   *
   * @param path - The directory path to validate as a project root. Can be
   * relative or absolute
   * @returns A RootPath instance if the path contains a `.siroot` file, or
   * a RootPathNotFoundError if it does not
   */
  static from(path: string): RootPath | RootPathNotFoundError {
    const candidateRootPath = Project.projectBasePath(path);
    const candidateRootMarkerPath = Project.projectMarkerPath(
      candidateRootPath,
    );

    if (candidateRootMarkerPath.existsSync()) {
      return new RootPath(dirname(candidateRootMarkerPath.toString()));
    } else {
      return new RootPathNotFoundError(candidateRootPath.toString());
    }
  }

  /**
   * Converts this RootPath to a Project instance.
   *
   * This is a convenience method that creates a Project instance using this
   * RootPath's validated path. The Project class provides utilities for working
   * with the project structure, including path generation for schemas, actions,
   * and other project components.
   *
   * @returns A Project instance initialized with this root path
   */
  toProject(): Project {
    return new Project(this.path);
  }
}

/**
 * Cliffy command type for parsing and validating RootPath arguments.
 *
 * This type integrates with Cliffy's command parsing system to automatically
 * validate that command-line path arguments point to valid si project
 * roots (directories containing a `.siroot` marker file).
 *
 * When used in a Cliffy command definition, this type:
 * 1. Accepts a path string from the command line
 * 2. Validates that the path contains a `.siroot` marker file
 * 3. Returns a RootPath instance if valid, or a RootPathNotFoundError if not
 *
 * ## Usage with Cliffy Commands
 *
 * Register this type with your command and use it in argument definitions:
 *
 * @example Basic usage
 * ```ts
 * import { Command } from "@cliffy/command";
 * import { RootPathType } from "./cli/root-path.ts";
 *
 * await new Command()
 *   .name("build")
 *   .type("rootpath", new RootPathType())
 *   .arguments("<root:rootpath>")
 *   .action((options, root) => {
 *     console.log(`Building project at: ${root.path}`);
 *     const project = root.toProject();
 *     // Use project...
 *   })
 *   .parse();
 * ```
 *
 * @example Optional root path argument
 * ```ts
 * await new Command()
 *   .name("list-schemas")
 *   .type("rootpath", new RootPathType())
 *   .arguments("[root:rootpath]")
 *   .action(async (options, root) => {
 *     const project = root
 *       ? root.toProject()
 *       : RootPath.findFromCwd().toProject();
 *     const schemas = await project.currentSchemaDirNames();
 *     console.log(schemas);
 *   })
 *   .parse();
 * ```
 *
 * @example With error handling
 * ```ts
 * import { RootPathNotFoundError } from "./cli/root-path.ts";
 *
 * await new Command()
 *   .name("deploy")
 *   .type("rootpath", new RootPathType())
 *   .arguments("<root:rootpath>")
 *   .action((options, root) => {
 *     if (root instanceof RootPathNotFoundError) {
 *       console.error(root.message);
 *       Deno.exit(1);
 *     }
 *     // Proceed with deployment...
 *   })
 *   .parse();
 * ```
 */
export class RootPathType extends Type<RootPath | RootPathNotFoundError> {
  /**
   * Parses and validates a command-line argument as a RootPath.
   *
   * This method is called automatically by Cliffy when parsing command-line
   * arguments. It validates that the provided path contains a `.siroot`
   * marker file.
   *
   * Note: This method does not search parent directories. It only validates
   * the exact path provided by the user. Use RootPath.find() if you need to
   * search upward through the directory tree.
   *
   * @param value - The ArgumentValue containing the command-line path string
   * @returns A RootPath instance if the path is valid, or a
   * RootPathNotFoundError if the path does not contain a `.siroot` file
   */
  public parse({ value }: ArgumentValue): RootPath | RootPathNotFoundError {
    return RootPath.from(value);
  }
}
