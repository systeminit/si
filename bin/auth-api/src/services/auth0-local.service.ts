/* eslint-disable no-console */
/**
 * Local development Auth0 mock service
 *
 * This service provides a local-only authentication bypass for development.
 * It replaces Auth0 calls with mock data when LOCAL_AUTH_MODE=true.
 *
 * NO REMOTE CALLS ARE MADE - all authentication is local.
 */

import { ApiResponse } from "auth0";
import type { GetUsers200ResponseOneOfInner as Auth0User } from "auth0";
import { ulid } from "ulidx";

const isLocalAuthMode = process.env.LOCAL_AUTH_MODE === "true";

// Hardcoded local development user configuration
const LOCAL_USER_EMAIL = "dev@systeminit.local";
const LOCAL_USER_FIRST_NAME = "Local";
const LOCAL_USER_LAST_NAME = "Developer";
const LOCAL_USER_NICKNAME = "localdev";

/**
 * Creates a mock Auth0 user profile for local development
 */
function createLocalAuth0Profile(auth0Id: string): ApiResponse<Auth0User> {
  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    level: "info",
    type: "local-auth",
    action: "create_mock_profile",
    auth0Id,
    email: LOCAL_USER_EMAIL,
    message: "🔧 LOCAL AUTH MODE: Creating mock Auth0 profile - NO REMOTE CALLS",
  }));

  const profile: Auth0User = {
    user_id: auth0Id,
    email: LOCAL_USER_EMAIL,
    email_verified: true, // Always verified in local mode
    nickname: LOCAL_USER_NICKNAME,
    given_name: LOCAL_USER_FIRST_NAME,
    family_name: LOCAL_USER_LAST_NAME,
    name: `${LOCAL_USER_FIRST_NAME} ${LOCAL_USER_LAST_NAME}`,
    picture: `https://ui-avatars.com/api/?name=${LOCAL_USER_FIRST_NAME}+${LOCAL_USER_LAST_NAME}&background=random`,
    updated_at: new Date().toISOString(),
    created_at: new Date().toISOString(),
  };

  return {
    data: profile,
    status: 200,
    statusText: "OK (Mock)",
    headers: {},
  } as ApiResponse<Auth0User>;
}

/**
 * Mock implementation of completeAuth0TokenExchange for local development
 * Bypasses Auth0 OAuth flow entirely
 */
export async function completeLocalAuth0TokenExchange(email?: string) {
  if (!isLocalAuthMode) {
    throw new Error("completeLocalAuth0TokenExchange can only be called when LOCAL_AUTH_MODE=true");
  }

  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    level: "info",
    type: "local-auth",
    action: "token_exchange",
    email: email || LOCAL_USER_EMAIL,
    message: "🔧 LOCAL AUTH MODE: Skipping Auth0 token exchange - using local credentials",
  }));

  // Generate a stable auth0Id based on email for consistency across restarts
  const auth0Id = `local|${Buffer.from(email || LOCAL_USER_EMAIL).toString('base64').replace(/=/g, '')}`;

  const profile = createLocalAuth0Profile(auth0Id);

  // Return mock access token (not actually used in local mode)
  const mockToken = `local_dev_token_${ulid()}`;

  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    level: "info",
    type: "local-auth",
    action: "token_exchange_complete",
    auth0Id,
    message: "🔧 LOCAL AUTH MODE: Token exchange complete - user authenticated locally",
  }));

  return { profile, token: mockToken };
}

/**
 * Mock implementation of fetchAuth0Profile for local development
 * Returns a mock profile without calling Auth0 Management API
 */
export async function fetchLocalAuth0Profile(auth0Id: string): Promise<ApiResponse<Auth0User>> {
  if (!isLocalAuthMode) {
    throw new Error("fetchLocalAuth0Profile can only be called when LOCAL_AUTH_MODE=true");
  }

  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    level: "info",
    type: "local-auth",
    action: "fetch_profile",
    auth0Id,
    message: "🔧 LOCAL AUTH MODE: Fetching mock profile - NO Auth0 Management API call",
  }));

  return createLocalAuth0Profile(auth0Id);
}

/**
 * Returns the mock logout URL for local development
 */
export function getLocalAuth0LogoutUrl(): string {
  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    level: "info",
    type: "local-auth",
    action: "logout_url",
    message: "🔧 LOCAL AUTH MODE: Generating local logout URL",
  }));

  return `${process.env.AUTH_PORTAL_URL}?logged_out=true`;
}

/**
 * Check if local auth mode is enabled
 */
export function isLocalAuth(): boolean {
  return isLocalAuthMode;
}

/**
 * Logs a warning if local auth mode is enabled at startup
 */
export function logLocalAuthWarning(): void {
  if (isLocalAuthMode) {
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "warn",
      type: "local-auth",
      action: "startup",
      message: "🔧🔧🔧 LOCAL AUTH MODE ENABLED 🔧🔧🔧",
      details: "Auth0 is BYPASSED - all authentication is local - DO NOT USE IN PRODUCTION",
      user: `${LOCAL_USER_EMAIL} (${LOCAL_USER_FIRST_NAME} ${LOCAL_USER_LAST_NAME})`,
      workspace: "Local Development @ http://localhost:8080",
    }));
  }
}
