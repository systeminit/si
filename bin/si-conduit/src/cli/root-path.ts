/**
 * Project root path discovery and validation utilities.
 *
 * This module provides functionality to find and validate project root
 * directories by searching for a `.conduitroot` marker file. It includes both a
 * RootPath class for programmatic use and a RootPathType for use with Cliffy
 * CLI commands.
 *
 * @module
 */

import { ArgumentValue, Type, ValidationError } from "@cliffy/command";
import { dirname, join } from "@std/path";
import { Project } from "../project.ts";

/** The marker file used to identify a conduit project root directory. */
const ROOT_MARKER = ".conduitroot";

/**
 * Represents a validated project root directory path.
 *
 * This class provides methods to validate and find project root directories,
 * either by directly validating a given path or by searching upwards from a
 * starting directory for a `.conduitroot` marker file.
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
   * @returns The RootPath instance for the project root
   * @throws Error if no project root is found
   */
  static findFromCwd(): RootPath {
    return this.find(Deno.cwd());
  }

  /**
   * Finds the project root by searching upwards from the given start directory
   * for a `.conduitroot` file.
   *
   * @param startDir - The directory to start searching from
   * @returns The absolute path to the project root
   * @throws Error if no project root is found
   */
  static find(startDir: string): RootPath {
    const start = startDir === "." ? Deno.cwd() : startDir;

    try {
      const stat = Deno.statSync(start);
      if (!stat.isDirectory) {
        throw new ValidationError(`not a directory: ${start}`);
      }
    } catch (err) {
      if (err instanceof ValidationError) {
        throw err;
      }
      if (err instanceof Deno.errors.NotFound) {
        throw new ValidationError(`directory not found: ${start}`);
      }
      throw err;
    }

    let currentDir = start;

    while (true) {
      const candidateRootMarkerPath = join(currentDir, ROOT_MARKER);

      try {
        const stat = Deno.statSync(candidateRootMarkerPath);
        if (stat.isFile) {
          return new RootPath(dirname(candidateRootMarkerPath));
        }
      } catch (err) {
        if (err instanceof Deno.errors.NotFound) {
          // File doesn't exist, continue searching
        } else {
          throw err;
        }
      }

      const parentDir = dirname(currentDir);

      // If we've reached the root of the filesystem, stop
      if (parentDir === currentDir) {
        throw new ValidationError(
          `no ${ROOT_MARKER} file found in ${start} or any parent directory`,
        );
      }

      currentDir = parentDir;
    }
  }

  /**
   * Validates and creates a RootPath from the given path.
   *
   * This method validates that the given path contains a `.conduitroot` marker
   * file.
   *
   * Unlike `find`, this method does not search parent directories.
   *
   * @param path - The directory path to validate as a project root
   * @returns The RootPath instance for the validated project root
   * @throws ValidationError if the path does not contain a `.conduitroot` file
   */
  static from(path: string): RootPath {
    const realPath = Deno.realPathSync(path);
    const candidateRootMarkerPath = join(realPath, ROOT_MARKER);

    try {
      const stat = Deno.statSync(candidateRootMarkerPath);
      if (stat.isFile) {
        return new RootPath(dirname(candidateRootMarkerPath));
      }
    } catch (err) {
      if (err instanceof Deno.errors.NotFound) {
        // File doesn't exist
      } else {
        throw err;
      }
    }

    throw new ValidationError(`no ${ROOT_MARKER} file found in ${realPath}`);
  }

  /**
   * Converts this RootPath to a Project instance.
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
 * This type can be used with Cliffy commands to automatically parse and
 * validate directory path arguments as project roots containing a
 * `.conduitroot` marker file.
 */
export class RootPathType extends Type<RootPath> {
  /**
   * Parses and validates a command-line argument as a RootPath.
   *
   * @param value - The command-line argument value to parse
   * @returns The validated RootPath instance
   * @throws ValidationError if the path does not contain a `.conduitroot` file
   */
  public parse({ value }: ArgumentValue): RootPath {
    return RootPath.from(value);
  }
}
