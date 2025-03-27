/**
 * Centralized OpenAI client for AI Agent
 *
 * This module provides a unified way to initialize and access the OpenAI client
 * throughout the application.
 *
 * @module
 */

import OpenAI from "jsr:@openai/openai@^4";

/**
 * Configuration options for the OpenAI client
 */
export interface OpenAIClientOptions {
  /**
   * API key to use for authentication
   * If not provided, will use the OPENAI_API_KEY environment variable
   */
  apiKey?: string;

  /**
   * Default model to use for completions if not specified
   * @default "gpt-4o-mini"
   */
  defaultModel?: string;
}

/**
 * Default model to use for OpenAI API calls
 */
export const DEFAULT_MODEL = "gpt-4o";

/**
 * Default temperature setting for deterministic, structured outputs like JSON
 * Lower values (0.0-0.2) reduce randomness and increase accuracy for schema adherence
 */
export const DEFAULT_SCHEMA_TEMPERATURE = 0.0;

/**
 * Singleton OpenAI client instance
 */
let client: OpenAI | null = null;

/**
 * Initialize the OpenAI client with the given options
 *
 * @param options Configuration options for the OpenAI client
 */
export function initClient(options: OpenAIClientOptions = {}): void {
  client = new OpenAI({
    apiKey: options.apiKey,
  });
}

/**
 * Get the OpenAI client instance
 * If the client has not been initialized, initializes it with default options
 *
 * @returns The OpenAI client instance
 */
export function getClient(): OpenAI {
  if (!client) {
    initClient();
  }
  return client!;
}

/**
 * Maximum content length allowed by OpenAI API
 * Set below the actual limit to maintain a safety buffer for other content.
 */
export const MAX_CONTENT_LENGTH = 250000;
