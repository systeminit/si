/**
 * Context Module - Global Application Context Management
 *
 * This module provides a singleton context that manages global application
 * state including logging and analytics services. The context must be
 * initialized once at application startup using {@link Context.init} before
 * accessing it via {@link Context.instance}.
 *
 * The singleton pattern is used here to ensure consistent logging and analytics
 * configuration across all commands and modules without explicit dependency
 * passing through every function call.
 *
 * @example
 * ```ts
 * import { Context } from "./context.ts";
 *
 * // Initialize once at startup
 * await Context.init({
 *   verbose: 2,
 *   noColor: false,
 *   userData: { userId: "123", workspaceId: "ws456" }
 * });
 *
 * // Access anywhere in the application
 * const ctx = Context.instance();
 * ctx.logger.info("Application started");
 * ctx.analytics.trackEvent("app-started");
 * ```
 *
 * @module
 */

import type { Logger } from "@logtape/logtape";
import { Analytics } from "./analytics.ts";
import type { UserData } from "./jwt.ts";
import {
  configureLogger,
  getLogger,
  isInteractive as loggerIsInteractive,
  type VerbosityLevel,
} from "./logger.ts";

/** Event prefix for all analytics events tracked by this application. */
const ANALYTICS_EVENT_PREFIX = "conduit";

/** Logger category for all application logs. */
const APP_CATEGORY = "si-conduit";

/**
 * Options for initializing the application context.
 */
export interface ContextOptions {
  /** Disable colored output in logs and prompts. */
  noColor?: boolean;
  /** User and workspace identification data for authenticated tracking. */
  userData?: UserData;
  /** Verbosity level for logging (0=errors only, 4=trace). */
  verbose?: number;
}

/**
 * Global application context managing logging and analytics services.
 *
 * This singleton class provides centralized access to logger and analytics
 * instances that are configured once at application startup. It uses the
 * singleton pattern to avoid passing dependencies through every function call.
 */
export class Context {
  private static context: Context;

  /** Logger instance for application logging. */
  public readonly logger: Logger;

  /** Analytics instance for event tracking. */
  public readonly analytics: Analytics;

  /** Whether or not the CLI is running in an interactive session. */
  public readonly isInteractive: boolean;

  private constructor(
    logger: Logger,
    analytics: Analytics,
    isInteractive: boolean,
  ) {
    this.logger = logger;
    this.analytics = analytics;
    this.isInteractive = isInteractive;
  }

  /**
   * Initializes the global application context.
   *
   * This method must be called once at application startup before any calls to
   * {@link instance}. It configures the logger and analytics services based on
   * the provided options.
   *
   * @param options - Configuration options for logging and analytics
   * @returns A promise resolving to the initialized Context instance
   *
   * @example
   * ```ts
   * // Initialize with defaults
   * await Context.init({});
   *
   * // Initialize with custom verbosity and user data
   * await Context.init({
   *   verbose: 3,
   *   userData: { userId: "user123", workspaceId: "ws456" }
   * });
   * ```
   */
  public static async init(options: ContextOptions): Promise<Context> {
    const verbosity = (options.verbose ?? 0) as VerbosityLevel;
    await configureLogger(verbosity, options.noColor);
    const logger = getLogger([APP_CATEGORY]);

    const analytics = new Analytics(ANALYTICS_EVENT_PREFIX, options.userData);

    const isInteractive = loggerIsInteractive();

    this.context = new Context(logger, analytics, isInteractive);

    return this.context;
  }

  /**
   * Returns the global context instance.
   *
   * This method provides access to the singleton context instance. It will
   * throw {@link ContextNotInitializedError} if called before {@link init}.
   *
   * @returns The initialized Context instance
   * @throws {ContextNotInitializedError} If context has not been initialized
   *
   * @example
   * ```ts
   * const ctx = Context.instance();
   * ctx.logger.info("Processing command");
   * ctx.analytics.trackEvent("command-executed");
   * ```
   */
  public static instance(): Context {
    if (this.context) {
      return this.context;
    } else {
      throw new ContextNotInitializedError();
    }
  }

  public static isInitialized(): boolean {
    if (this.context) {
      return true;
    } else {
      return false;
    }
  }
}

/**
 * Error thrown when attempting to access Context before initialization.
 */
export class ContextNotInitializedError extends Error {
  constructor() {
    super(
      "Context has not been initialized. Call Context.init() before accessing Context.instance().",
    );
    this.name = "ContextNotInitializedError";
  }
}
