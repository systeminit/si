/**
 * Logger Module - LogTape Configuration for SI
 *
 * This module configures LogTape for CLI logging with automatic TTY detection,
 * ANSI coloring for interactive terminals, and clean output for non-interactive
 * use.
 *
 * @module
 */

import {
  configure,
  getConsoleSink,
  getLogger as logtapeGetLogger,
  type Logger,
  type LogLevel,
  type LogRecord,
} from "@logtape/logtape";
import { getPrettyFormatter } from "@logtape/pretty";
import {
  DEFAULT_REDACT_FIELDS,
  JWT_PATTERN,
  redactByField,
  redactByPattern,
} from "@logtape/redaction";

/** Application logger category for all log messages. */
const APP_LOGGER_CATEGORY = "si";

/** Width allocated for category names in pretty-printed logs. */
const CATEGORY_WIDTH = 15;

/** Log level for internal LogTape meta messages. */
const META_LOGGER_LEVEL: LogLevel = "warning";

/**
 * Verbosity level for logging.
 *
 * Controls the minimum log level that will be displayed:
 * - **0**: Errors only
 * - **1**: Errors and warnings
 * - **2**: Errors, warnings, and info (default for most commands)
 * - **3**: Errors, warnings, info, and debug
 * - **4**: All messages including trace
 *
 * @example
 * ```ts
 * // Minimal logging (errors only)
 * await configureLogger(0);
 *
 * // Standard logging (info and above)
 * await configureLogger(2);
 *
 * // Debug logging
 * await configureLogger(3);
 * ```
 */
export type VerbosityLevel = 0 | 1 | 2 | 3 | 4;

/**
 * Converts verbosity level to LogTape log level.
 *
 * Maps numeric verbosity levels to LogTape's string-based log levels.
 *
 * @param verbosity - The numeric verbosity level (0-4)
 * @returns The corresponding LogTape log level
 *
 * @internal
 */
function verbosityToLogLevel(verbosity: VerbosityLevel): LogLevel {
  switch (verbosity) {
    case 0:
      return "error";
    case 1:
      return "warning";
    case 2:
      return "info";
    case 3:
      return "debug";
    case 4:
      return "trace";
  }
}

/**
 * Checks if output is to an interactive TTY.
 *
 * Uses Deno's `isTerminal()` to detect if stdout is connected to an interactive
 * terminal (TTY). Returns false when output is piped or redirected.
 *
 * @returns True if stdout is an interactive terminal
 *
 * @internal
 */
export function isInteractive(): boolean {
  return Deno.stdout.isTerminal();
}

/**
 * Checks if colors should be used based on NO_COLOR environment variable and
 * CLI option.
 *
 * Follows the NO_COLOR standard: if the NO_COLOR environment variable is set
 * (to any value), colors should be disabled. The CLI option takes precedence.
 *
 * @param noColorOption - CLI option to disable colors (takes precedence over
 *   env var)
 * @returns True if colors should be used, false otherwise
 *
 * @see {@link https://no-color.org/}
 * @internal
 */
function shouldUseColors(noColorOption?: boolean): boolean {
  if (noColorOption === true) {
    return false;
  }
  // deno-lint-ignore si-rules/no-deno-env-get -- Used only to detect NO_COLOR
  const noColor = Deno.env.get("NO_COLOR");

  // Per NO_COLOR spec, presence of the variable (not its value) disables colors
  return noColor === undefined;
}

/**
 * Creates a formatter for non-interactive (non-TTY) output.
 *
 * Produces plain text log output with ISO 8601 timestamps, uppercase log
 * levels, and space-joined message parts. Suitable for log files, CI/CD
 * environments, and piped output.
 *
 * @param record - The log record to format
 * @returns Formatted log string
 *
 * @example
 * Output format: `[2025-01-15T10:30:45.123Z] INFO: Application started`
 *
 * @internal
 */
function createNonInteractiveFormatter(record: LogRecord): string {
  // LogRecord.timestamp is a number (milliseconds since epoch)
  const timestamp = new Date(record.timestamp).toISOString();
  return `[${timestamp}] ${String(record.level).toUpperCase()}: ${
    record.message.join(
      " ",
    )
  }`;
}

/**
 * Configures the LogTape logger with appropriate formatters based on TTY
 * detection.
 *
 * This function sets up logging with automatic detection of terminal
 * capabilities:
 * - **Interactive (TTY)**: Pretty-printed output with colors (unless disabled)
 * - **Non-interactive (piped/redirected)**: Plain text with ISO timestamps
 *
 * The logger automatically redacts sensitive data including JWT tokens and API
 * tokens from log output.
 *
 * @param verbosity - The verbosity level (0-4), defaults to 0 (errors only)
 * @param noColor - CLI option to disable colors (takes precedence over NO_COLOR
 *   env var)
 * @returns A promise that resolves when logging is configured
 *
 * @example
 * ```ts
 * // Configure with default verbosity (errors only)
 * await configureLogger();
 *
 * // Configure with info level logging
 * await configureLogger(2);
 *
 * // Configure with debug logging and no colors
 * await configureLogger(3, true);
 * ```
 */
export async function configureLogger(
  verbosity: VerbosityLevel = 0,
  noColor?: boolean,
): Promise<void> {
  const logLevel = verbosityToLogLevel(verbosity);
  const interactive = isInteractive();
  const useColors = shouldUseColors(noColor);

  const formatter = interactive
    ? getPrettyFormatter({
      messageStyle: null,
      categoryStyle: "italic",
      categoryWidth: CATEGORY_WIDTH,
      colors: useColors,
    })
    : createNonInteractiveFormatter;

  await configure({
    sinks: {
      console: redactByField(
        getConsoleSink({
          formatter: redactByPattern(formatter, [JWT_PATTERN]),
        }),
        ["apiToken", ...DEFAULT_REDACT_FIELDS],
      ),
    },
    filters: {},
    loggers: [
      {
        category: [APP_LOGGER_CATEGORY],
        lowestLevel: logLevel,
        sinks: ["console"],
      },
      // Configure the meta logger to suppress internal LogTape messages
      {
        category: ["logtape", "meta"],
        lowestLevel: META_LOGGER_LEVEL,
        sinks: ["console"],
      },
    ],
  });
}

export function getLogger(category?: string | readonly string[]): Logger {
  return logtapeGetLogger([APP_LOGGER_CATEGORY, ...(category ?? [])]);
}
