/**
 * Test utilities for Luminork API tests
 */

import { load as loadEnv } from 'https://deno.land/std@0.220.1/dotenv/mod.ts';
import { LuminorkApi } from './api/index.ts';
import { ConfigError } from './client.ts';

// Re-export ConfigError
export { ConfigError } from './client.ts';

// Test configuration
export interface TestConfig {
  baseUrl: string;
  authToken: string;
  workspaceId: string;
  timeout: number;
}

/**
 * Load test configuration with proper fallback order:
 * 1. Environment variables
 * 2. .env file variables
 * 3. CLI parameters (passed as function arguments)
 * 4. Default values
 */
export async function loadConfig(cliParams: {
  apiUrl?: string;
  authToken?: string;
  workspaceId?: string;
  timeout?: number;
} = {}): Promise<TestConfig> {
  // Original environment variables (before .env loading)
  const envApiUrl = Deno.env.get('LUMINORK_API_URL') || Deno.env.get('API_URL');
  const envAuthToken = Deno.env.get('LUMINORK_AUTH_TOKEN') || Deno.env.get('AUTH_TOKEN');
  const envWorkspaceId = Deno.env.get('LUMINORK_WORKSPACE_ID') || Deno.env.get('WORKSPACE_ID');
  const envTimeout = Deno.env.get('LUMINORK_TIMEOUT');

  // Try to load from .env file, but don't export to override existing environment variables
  const envFileVars: Record<string, string> = {};
  try {
    // Use load() with export:false to not override existing environment variables
    const dotEnvVars = await loadEnv({ export: false });
    Object.assign(envFileVars, dotEnvVars);
    console.log('Loaded variables from .env file');
  } catch (e) {
    const errorMessage = e instanceof Error ? e.message : String(e);
    console.log('No .env file found or error loading it:', errorMessage);
  }

  // Use environment variables first, then .env file, then CLI params, then defaults
  const baseUrl = envApiUrl ||
    envFileVars['LUMINORK_API_URL'] ||
    envFileVars['API_URL'] ||
    cliParams.apiUrl ||
    'http://localhost:5380';

  // Auth token with same priority order
  let authToken = envAuthToken ||
    envFileVars['LUMINORK_AUTH_TOKEN'] ||
    envFileVars['AUTH_TOKEN'] ||
    cliParams.authToken ||
    '';

  // Workspace ID with same priority order
  let workspaceId = envWorkspaceId ||
    envFileVars['LUMINORK_WORKSPACE_ID'] ||
    envFileVars['WORKSPACE_ID'] ||
    cliParams.workspaceId ||
    '';

  // Timeout with same priority order
  const timeoutValue = envTimeout ||
    envFileVars['LUMINORK_TIMEOUT'] ||
    (cliParams.timeout !== undefined ? String(cliParams.timeout) : null) ||
    '30000';

  const timeout = parseInt(timeoutValue, 10);

  // Remove quotes if present
  if (authToken.startsWith('"') && authToken.endsWith('"')) {
    authToken = authToken.slice(1, -1);
  }

  if (workspaceId.startsWith('"') && workspaceId.endsWith('"')) {
    workspaceId = workspaceId.slice(1, -1);
  }

  // Log configuration sources
  console.log('Configuration sources:');
  console.log(
    `- API URL: ${
      envApiUrl
        ? 'Environment'
        : envFileVars['LUMINORK_API_URL']
        ? '.env file'
        : cliParams.apiUrl
        ? 'CLI param'
        : 'Default'
    }`,
  );
  console.log(
    `- Auth Token: ${
      envAuthToken
        ? 'Environment'
        : envFileVars['LUMINORK_AUTH_TOKEN']
        ? '.env file'
        : cliParams.authToken
        ? 'CLI param'
        : 'Not set'
    }`,
  );
  console.log(
    `- Workspace ID: ${
      envWorkspaceId
        ? 'Environment'
        : envFileVars['LUMINORK_WORKSPACE_ID']
        ? '.env file'
        : cliParams.workspaceId
        ? 'CLI param'
        : 'Not set'
    }`,
  );
  console.log(
    `- Timeout: ${
      envTimeout
        ? 'Environment'
        : envFileVars['LUMINORK_TIMEOUT']
        ? '.env file'
        : cliParams.timeout !== undefined
        ? 'CLI param'
        : 'Default'
    }`,
  );

  // Validate required configuration - fail if missing
  if (!authToken) {
    throw new ConfigError(
      'LUMINORK_AUTH_TOKEN is required but not set in environment, .env file, or CLI parameters',
    );
  }

  if (!workspaceId) {
    throw new ConfigError(
      'LUMINORK_WORKSPACE_ID is required but not set in environment, .env file, or CLI parameters',
    );
  }

  return {
    baseUrl,
    authToken,
    workspaceId,
    timeout,
  };
}

/**
 * Create a configured Luminork API client for testing
 *
 * @param cliParams Optional CLI parameters that can override config in this order:
 *                 1. Environment variables
 *                 2. .env file
 *                 3. These CLI parameters
 *                 4. Default values
 */
export async function createTestClient(cliParams: {
  apiUrl?: string;
  authToken?: string;
  workspaceId?: string;
  timeout?: number;
} = {}): Promise<{ api: LuminorkApi; config: TestConfig }> {
  try {
    // Pass CLI parameters to loadConfig
    const config = await loadConfig(cliParams);

    const api = new LuminorkApi({
      baseUrl: config.baseUrl,
      timeout: config.timeout,
      authToken: config.authToken,
    });

    return { api, config };
  } catch (error) {
    if (error instanceof ConfigError) {
      console.error(`Configuration error: ${error.message}`);
      console.error(
        'Make sure to set configuration via environment variables, .env file, or CLI parameters',
      );
      throw error;
    }
    throw error;
  }
}

/**
 * Generate a unique test name with timestamp
 */
export function generateTestName(prefix = 'test'): string {
  const timestamp = new Date().toISOString().replace(/[^0-9]/g, '').slice(0, 14);
  return `${prefix}_${timestamp}`;
}

/**
 * Sleep for a specified number of milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retry a function multiple times with a delay between attempts
 */
export async function retry<T>(
  fn: () => Promise<T>,
  options: {
    maxAttempts?: number;
    delayMs?: number;
    onRetry?: (attempt: number, error: Error) => void;
  } = {},
): Promise<T> {
  const { maxAttempts = 3, delayMs = 1000, onRetry } = options;

  let lastError: Error | undefined;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      if (attempt < maxAttempts) {
        if (onRetry) {
          onRetry(attempt, lastError);
        }
        await sleep(delayMs);
      }
    }
  }

  throw lastError;
}

/**
 * Clean up resources created during tests
 */
export async function cleanupTestResources(
  api: LuminorkApi,
  workspaceId: string,
  changeSetIds: string[],
): Promise<void> {
  // In a test environment, using purge is safer than trying to delete individual change sets
  try {
    // First try purge operation if available
    await api.changeSets.purgeOpenChangeSets(workspaceId);
  } catch (error) {
    console.warn(`Purge operation failed: ${error}`);

    // Fall back to individual deletion
    for (const changeSetId of changeSetIds) {
      try {
        await api.changeSets.deleteChangeSet(workspaceId, changeSetId);
      } catch (error) {
        console.warn(`Failed to delete change set ${changeSetId}: ${error}`);
      }
    }
  }
}
