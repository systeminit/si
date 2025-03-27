/**
 * Logging utilities for the CloudFormation database module.
 * 
 * This module configures and exports a logger instance that provides
 * standardized logging capabilities across the library.
 * 
 * @module logger
 */

import adze, { setup } from "npm:adze@2.2.1";

// Use environment variable for log level or default to "info"
const activeLevel = Deno.env.get("LOG_LEVEL") ?? "info";

// Configure the logger with timestamp and metadata
setup({
  // @ts-ignore Yeah yeah, it's okay - we know they could use a bad level
  activeLevel,
  meta: {
    "si": "is fun",
  },
});

// Create a sealed logger instance with emoji support and timestamps
const logger = adze.withEmoji.timestamp.seal();
export default logger;
