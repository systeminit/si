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
import { Configuration } from "@systeminit/api-client";
import axios from "axios";
import { Analytics } from "./analytics.ts";
import type { UserData } from "./cli/jwt.ts";
import {
  configureLogger,
  getLogger,
  isInteractive as loggerIsInteractive,
  type VerbosityLevel,
} from "./logger.ts";
import type { GlobalOptions } from "./cli.ts";
import { type Config, extractConfig } from "./cli/config.ts";
import * as jwt from "./cli/jwt.ts";
import { getUserAgent } from "./user_agent.ts";
import type { AxiosResponse } from "axios";
import type { AxiosRequestConfig } from "axios";

// Extend axios types to include metadata for request timing
declare module "axios" {
  export interface InternalAxiosRequestConfig {
    metadata?: {
      startTime: number;
    };
  }
}

/** Event prefix for all analytics events tracked by this application. */
const ANALYTICS_EVENT_PREFIX = "si";

/** Logger category for all application logs. */
const _APP_CATEGORY = "si";

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
  /** Auth API Url **/
  authApiUrl?: string;
  /** Auth API token */
  authApiToken?: string;
  /** System Initiative API base url **/
  baseUrl?: string;
  /** System Initiative API token */
  apiToken?: string;
  /** Current Workspace ID */
  workspaceId?: string;
  /** Current User ID */
  userId?: string;
}

/**
 * Global application context managing logging and analytics services.
 *
 * This singleton class provides centralized access to logger and analytics
 * instances that are configured once at application startup. It uses the
 * singleton pattern to avoid passing dependencies through every function call.
 */
export class Context {
  private static context?: Context;

  /** Logger instance for application logging. */
  public readonly logger: Logger;

  /** Analytics instance for event tracking. */
  public readonly analytics: Analytics;

  /** Whether or not the CLI is running in an interactive session. */
  public readonly isInteractive: boolean;

  /** Auth API url **/
  public readonly authApiUrl: string;

  /** System Initiative API base url **/
  public readonly baseUrl?: string;

  /** System Initiative workspace level API token **/
  public readonly apiToken?: string;

  /** System Initiative auth API level API token **/
  public readonly authApiToken?: string;

  /** Current Workspace ID (if initialized). */
  public readonly workspaceId?: string;

  /** Current User ID (if initialized). */
  public readonly userId?: string;

  private constructor(
    logger: Logger,
    analytics: Analytics,
    isInteractive: boolean,
    authApiUrl: string,
    authApiToken?: string,
    baseUrl?: string,
    apiToken?: string,
    workspaceId?: string,
    userId?: string,
  ) {
    this.logger = logger;
    this.analytics = analytics;
    this.isInteractive = isInteractive;
    this.authApiUrl = authApiUrl;
    this.authApiToken = authApiToken;
    this.baseUrl = baseUrl;
    this.apiToken = apiToken;
    this.workspaceId = workspaceId;
    this.userId = userId;
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
    if (typeof this.context?.logger === "undefined") {
      await configureLogger(verbosity, options.noColor);
    }

    const logger = getLogger();
    const analytics = new Analytics(ANALYTICS_EVENT_PREFIX, options.userData);

    const isInteractive = loggerIsInteractive();

    this.context = new Context(
      logger,
      analytics,
      isInteractive,
      options.authApiUrl ?? "https://auth-api.systeminit.com",
      options.authApiToken,
      options.baseUrl,
      options.apiToken,
      options.workspaceId,
      options.userId,
    );

    return this.context;
  }

  public static async initFromConfig(options: GlobalOptions): Promise<Context> {
    const authApiUrl = options.authApiUrl;
    const baseUrlOverride = options.baseUrl;
    const apiTokenOverride = options.apiToken;

    // Scaffold the api client configuration here based on the stored
    // tokens, or command line or env variable overrides.
    let config: Config | undefined;
    try {
      // Try to get token from stored config or environment variables
      config = extractConfig(authApiUrl);
    } catch (_error) {
      // If token decode fails or config extraction fails, just continue
      // without user data. this allows commands to run without
      // authentication when appropriate
      config = undefined;
    }
    if (!config) {
      config = {
        baseUrl: baseUrlOverride ?? "https://api.systeminit.com",
        authApiUrl,
      };
    }

    config.apiToken = apiTokenOverride ?? config.apiToken;
    config.baseUrl = baseUrlOverride ?? config.baseUrl;
    if (!config.baseUrl) {
      config.baseUrl = "https://api.systeminit.com";
    }

    const userData = config?.apiToken
      ? jwt.getUserDataFromToken(config?.apiToken)
      : undefined;

    return await Context.init({
      ...options,
      baseUrl: config.baseUrl,
      apiToken: config.apiToken,
      workspaceId: userData?.workspaceId,
      userId: userData?.userId,
      authApiUrl,
      userData,
    });
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

  public static workspaceId(): string {
    const ctx = Context.instance();
    if (!ctx.workspaceId) {
      throw new Error(
        "No workspace id configured. Run `si login` or set SI_API_TOKEN and SI_BASE_URL",
      );
    }
    return ctx.workspaceId;
  }

  public static userId(): string {
    const ctx = Context.instance();
    if (!ctx.userId) {
      throw new Error(
        "No user id configured. Run `si login` or set SI_API_TOKEN and SI_BASE_URL",
      );
    }
    return ctx.userId;
  }

  private static interceptorsAdded = false;

  public static apiConfig(): Configuration {
    const ctx = Context.instance();
    if (!ctx.apiToken) {
      throw new Error(
        "No API Context configured! Run `si login` or set SI_API_TOKEN",
      );
    }

    // Add global interceptors once
    addInterceptors();

    return new Configuration({
      basePath: ctx.baseUrl,
      accessToken: ctx.apiToken,
      baseOptions: {
        headers: {
          Authorization: `Bearer ${ctx.apiToken}`,
          "User-Agent": getUserAgent(),
        },
      },
    });
  }
}

/// Global flag to ensure interceptors are only added once
let INTERCEPTORS_ADDED = false;

function addInterceptors() {
  if (INTERCEPTORS_ADDED) return;
  INTERCEPTORS_ADDED = true;

  const logger = getLogger(["api"]);

  axios.interceptors.request.use((config) => {
    config.metadata = { startTime: Date.now() };
    const fullUrl = `${config.baseURL || ""}${config.url || ""}`;
    logger.trace(`API Request: ${config.method?.toUpperCase()} ${fullUrl}`);

    if (config.params && Object.keys(config.params).length > 0) {
      const paramStr = Object.entries(config.params)
        .map(([k, v]) => `${k}=${v}`)
        .join(", ");
      logger.trace(`  Query Params: ${paramStr}`);
    }

    // Log request body
    printPayload(config);

    return config;
  });

  axios.interceptors.response.use(
    (response) => {
      const duration = response.config.metadata?.startTime
        ? Date.now() - response.config.metadata.startTime
        : null;
      const fullUrl = `${response.config.baseURL || ""}${
        response.config.url || ""
      }`;

      logger.trace(
        `API Response: ${response.config.method?.toUpperCase()} ${fullUrl} → ${response.status} ${response.statusText}${
          duration ? ` (${duration}ms)` : ""
        }`,
      );

      printPayload(response);

      return response;
    },
    (error) => {
      const duration = error.config?.metadata?.startTime
        ? Date.now() - error.config.metadata.startTime
        : null;
      const fullUrl = `${error.config?.baseURL || ""}${
        error.config?.url || ""
      }`;

      logger.trace(
        `API Response Error: ${error.config?.method?.toUpperCase()} ${fullUrl} → ${
          error.response?.status || "No Status"
        } ${error.response?.statusText || error.message}${
          duration ? ` (${duration}ms)` : ""
        }`,
      );

      printPayload(error.response);

      return Promise.reject(error);
    },
  );

  function printPayload(response: AxiosResponse | AxiosRequestConfig) {
    if (response.data) {
      switch (typeof response.data) {
        case "object": {
          const dataEntries = Object.entries(response.data);
          for (const [k, v] of dataEntries) {
            if (Array.isArray(v)) {
              logger.trace(`    ${k}: [${v.length} items]`);
              // Show first few items of array
              v.slice(0, 3).forEach((item, idx) => {
                if (typeof item === "object") {
                  const itemEntries = Object.entries(item);
                  const itemStr = itemEntries
                    .map(([ik, iv]) => `${ik}=${JSON.stringify(iv)}`)
                    .join(", ");
                  logger.trace(`      [${idx}]: ${itemStr}`);
                } else {
                  logger.trace(`      [${idx}]: ${JSON.stringify(item)}`);
                }
              });
              if (v.length > 3) {
                logger.trace(`      ... ${v.length - 3} more items`);
              }
            } else {
              try {
                logger.trace(`    ${k}: ${JSON.stringify(v)}`);
              } catch {
                logger.trace(`    ${k}: [object]`);
              }
            }
          }
          break;
        }
        case "string": {
          for (const line of response.data.split("\n")) {
            logger.trace(`    ${line}`);
          }
          break;
        }
        default: {
          logger.trace(`    [${typeof response.data} ${response.data}]`);
        }
      }
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
