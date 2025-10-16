/**
 * JWT Module - JSON Web Token Decoding Utilities
 *
 * This module provides utilities for decoding JWT (JSON Web Token) tokens
 * to extract user and workspace identification data. It performs base64url
 * decoding of JWT payloads without signature verification.
 *
 * **Security Note:** This module only decodes JWTs and does NOT verify
 * cryptographic signatures. Token validation should be performed by the
 * backend API. Never trust decoded token data without proper verification.
 *
 * @example
 * ```ts
 * import { getUserDataFromToken } from "./jwt.ts";
 *
 * const token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
 * const userData = getUserDataFromToken(token);
 *
 * if (userData) {
 *   console.log(`User: ${userData.userId}, Workspace: ${userData.workspaceId}`);
 * }
 * ```
 *
 * @module
 */

import { unknownValueToErrorMessage } from "./helpers.ts";

/**
 * User identification data extracted from a JWT token.
 */
export interface UserData {
  /** Unique identifier for the workspace. */
  workspaceId: string;
  /** Unique identifier for the user. */
  userId: string;
}

/**
 * JWT payload structure with expected fields.
 *
 * This interface defines the expected structure of decoded JWT payloads
 * used by this application. Additional fields may be present in the token.
 */
interface JWTPayload {
  /** Workspace identifier from the token. */
  workspaceId?: unknown;
  /** User identifier from the token. */
  userId?: unknown;
  /** Standard JWT claims and any additional fields. */
  [key: string]: unknown;
}

/**
 * Extracts user and workspace data from a JWT token.
 *
 * This function decodes a JWT token and extracts the user and workspace
 * identification fields. It validates that the required fields are present
 * and are strings before returning.
 *
 * @param apiToken - The JWT token string to decode (optional)
 * @returns UserData object if token is valid and contains required fields,
 *   undefined if token is not provided
 * @throws {JWTDecodeError} If the token cannot be decoded or is missing required fields
 *
 * @example
 * ```ts
 * // With a valid token
 * const userData = getUserDataFromToken(token);
 * if (userData) {
 *   console.log(userData.userId, userData.workspaceId);
 * }
 *
 * // Handling errors
 * try {
 *   const userData = getUserDataFromToken(invalidToken);
 * } catch (error) {
 *   if (error instanceof JWTDecodeError) {
 *     console.error("Invalid token:", error.message);
 *   }
 * }
 * ```
 */
export function getUserDataFromToken(apiToken?: string): UserData | undefined {
  if (!apiToken) {
    return undefined;
  }

  // Trim whitespace that might have been accidentally included
  const trimmedToken = apiToken.trim();
  if (trimmedToken.length === 0) {
    return undefined;
  }

  const payload = decodeJWT(trimmedToken);

  // Validate that required fields exist and are strings
  if (typeof payload.workspaceId !== "string") {
    throw new JWTDecodeError(
      `Token payload is missing required field 'workspaceId' or it is not a string`,
    );
  }

  if (typeof payload.userId !== "string") {
    throw new JWTDecodeError(
      `Token payload is missing required field 'userId' or it is not a string`,
    );
  }

  return {
    workspaceId: payload.workspaceId,
    userId: payload.userId,
  };
}

/**
 * Error thrown when JWT decoding fails.
 */
export class JWTDecodeError extends Error {
  constructor(message: string, cause?: Error) {
    super(message);
    this.name = "JWTDecodeError";
    if (cause) {
      this.cause = cause;
    }
  }
}

/**
 * Error thrown when JWT format is invalid.
 */
export class InvalidJWTFormatError extends JWTDecodeError {
  constructor(reason: string) {
    super(`Invalid JWT format: ${reason}`);
    this.name = "InvalidJWTFormatError";
  }
}

/**
 * Decodes a JWT token and returns its payload.
 *
 * This function performs base64url decoding of the JWT payload section
 * without verifying the cryptographic signature. JWTs consist of three
 * base64url-encoded parts separated by dots: header.payload.signature.
 *
 * @param token - The JWT token string to decode
 * @returns The decoded JWT payload as a record
 * @throws {InvalidJWTFormatError} If the token format is invalid
 * @throws {JWTDecodeError} If decoding or parsing fails
 *
 * @internal
 */
function decodeJWT(token: string): JWTPayload {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) {
      throw new InvalidJWTFormatError(
        `Expected 3 parts separated by dots (header.payload.signature), got ${parts.length}`,
      );
    }

    const [, payloadPart] = parts;

    if (!payloadPart || payloadPart.length === 0) {
      throw new InvalidJWTFormatError("Payload part is empty");
    }

    // Convert base64url to base64 (replace URL-safe chars)
    const base64 = payloadPart.replace(/-/g, "+").replace(/_/g, "/");

    // Decode base64 to string
    let decoded: string;
    try {
      decoded = atob(base64);
    } catch (error) {
      throw new InvalidJWTFormatError(
        `Invalid base64url encoding: ${unknownValueToErrorMessage(error)}`,
      );
    }

    // Parse JSON payload
    try {
      return JSON.parse(decoded) as JWTPayload;
    } catch (error) {
      throw new JWTDecodeError(
        `Payload is not valid JSON: ${unknownValueToErrorMessage(error)}`,
        error instanceof Error ? error : undefined,
      );
    }
  } catch (error) {
    // Re-throw our custom errors as-is
    if (error instanceof JWTDecodeError) {
      throw error;
    }

    // Wrap unexpected errors
    throw new JWTDecodeError(
      `Failed to decode JWT: ${unknownValueToErrorMessage(error)}`,
      error instanceof Error ? error : undefined,
    );
  }
}
