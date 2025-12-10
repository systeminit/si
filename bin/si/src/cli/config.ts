/**
 * CLI configuration utilities
 *
 * This module provides utilities for extracting configuration from environment variables.
 *
 * Directory structure for storing authentication data:
 * {config}/auth/
 * ├── current_user          # ID of the current active user
 * ├── current_workspace     # ID of the current active workspace
 * └── {userId}/             # Directory for each user (named by user ID)
 *     ├── user              # user details (json)
 *     ├── token             # The user's auth API token (not workspace bound)
 *     └── {workspaceId}/    # Directory for each workspace (named by workspace ID)
 *         ├── workspace     # workspace details (json)
 *         └── token         # workspace-specific automation token for api requests
 */

import { join } from "@std/path";
import envPaths from "npm:env-paths";
import type { UserDetails, WorkspaceDetails } from "./auth.ts";

const AUTH_FOLDER = "auth";
const CURRENT_USER_FILE = "current_user";
const CURRENT_WORKSPACE_FILE = "current_workspace";
const USER_FILE = "user";
const WORKSPACE_FILE = "workspace";
const TOKEN_FILE = "token";

const configPath = () => envPaths("si").config;
const authPath = () => join(configPath(), AUTH_FOLDER);

const currentUserPath = () =>
  join(configPath(), AUTH_FOLDER, CURRENT_USER_FILE);
const currentWorkspacePath = () =>
  join(configPath(), AUTH_FOLDER, CURRENT_WORKSPACE_FILE);

const ensureAuthFolder = () => {
  Deno.mkdirSync(authPath(), { recursive: true, mode: 0o700 });
};

const ensureUserFolder = (userId: string) => {
  const auth = authPath();
  const userFolder = join(auth, userId);
  Deno.mkdirSync(userFolder, { recursive: true, mode: 0o700 });
};

const ensureWorkspaceFolder = (userId: string, workspaceId: string) => {
  Deno.mkdirSync(join(authPath(), userId, workspaceId), {
    recursive: true,
    mode: 0o700,
  });
};

export const setCurrentUser = (userId: string) => {
  ensureAuthFolder();
  Deno.writeTextFileSync(currentUserPath(), userId, {
    create: true,
    mode: 0o600,
  });
};

export const setCurrentWorkspace = (workspaceId: string) => {
  ensureAuthFolder();
  Deno.writeTextFileSync(currentWorkspacePath(), workspaceId, {
    create: true,
    mode: 0o600,
  });
};

export const getCurrentUser = (): string | undefined => {
  try {
    const userId = Deno.readTextFileSync(currentUserPath());
    return userId.trim();
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return undefined;
    }
    throw error;
  }
};

export const getCurrentWorkspace = (): string | undefined => {
  try {
    const workspaceId = Deno.readTextFileSync(currentWorkspacePath());
    return workspaceId.trim();
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return undefined;
    }
    throw error;
  }
};

export const writeUser = (userDetails: UserDetails, authApiToken: string) => {
  ensureUserFolder(userDetails.id);
  Deno.writeTextFileSync(
    join(authPath(), userDetails.id, USER_FILE),
    JSON.stringify(userDetails),
    { create: true, mode: 0o600 },
  );
  Deno.writeTextFileSync(
    join(authPath(), userDetails.id, TOKEN_FILE),
    authApiToken,
    { create: true, mode: 0o600 },
  );
};

export const writeWorkspace = (
  userId: string,
  workspaceDetails: WorkspaceDetails,
  workspaceToken: string,
) => {
  ensureWorkspaceFolder(userId, workspaceDetails.id);
  Deno.writeTextFileSync(
    join(authPath(), userId, workspaceDetails.id, WORKSPACE_FILE),
    JSON.stringify(workspaceDetails),
    { create: true, mode: 0o600 },
  );
  Deno.writeTextFileSync(
    join(authPath(), userId, workspaceDetails.id, TOKEN_FILE),
    workspaceToken,
    { create: true, mode: 0o600 },
  );
};

export const getUserDetails = (
  userId: string,
): { userDetails?: UserDetails; token?: string } => {
  const userDetailsPath = join(authPath(), userId, USER_FILE);
  const tokenPath = join(authPath(), userId, TOKEN_FILE);

  let userDetails: UserDetails;
  try {
    const userDetailsContent = Deno.readTextFileSync(userDetailsPath);
    userDetails = JSON.parse(userDetailsContent);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return { userDetails: undefined, token: undefined };
    }
    throw error;
  }

  let token: string | undefined;
  try {
    token = Deno.readTextFileSync(tokenPath);
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw error;
    }
  }

  return { userDetails, token };
};

export const getWorkspaceDetails = (
  userId: string,
  workspaceId: string,
): { workspaceDetails?: WorkspaceDetails; token?: string } => {
  const workspaceDetailsPath = join(
    authPath(),
    userId,
    workspaceId,
    WORKSPACE_FILE,
  );
  const tokenPath = join(authPath(), userId, workspaceId, TOKEN_FILE);

  let workspaceDetails: WorkspaceDetails;
  try {
    const workspaceDetailsContent = Deno.readTextFileSync(workspaceDetailsPath);
    workspaceDetails = JSON.parse(workspaceDetailsContent);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return { workspaceDetails: undefined, token: undefined };
    }
    throw error;
  }

  let token: string | undefined;
  try {
    token = Deno.readTextFileSync(tokenPath);
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw error;
    }
  }

  workspaceDetails.baseUrl = getApiUrlForInstance(workspaceDetails.instanceUrl);

  return { workspaceDetails, token };
};

export interface Config {
  baseUrl?: string;
  authApiUrl?: string;
  authApiToken?: string;
  apiToken?: string;
}

export function extractConfig(authApiUrl: string): Config {
  // Try to get configuration from stored settings first
  let config: Config | undefined;
  if (hasStoredConfig()) {
    try {
      config = extractConfigFromStore(authApiUrl);
    } catch (error) {
      // If we have stored config but can't extract it, log the error
      console.warn("Failed to extract stored configuration:", error);
    }
  }

  return config ?? {};
}

export function extractConfigFromStore(authApiUrl: string): Config {
  const userId = getCurrentUser();
  if (!userId) {
    throw new Error(
      "No user logged in. Please run 'si login' to authenticate with System Initiative.",
    );
  }

  const { token: authApiToken } = getUserDetails(userId);

  const workspaceId = getCurrentWorkspace();
  if (!workspaceId) {
    // User is logged in but hasn't selected a workspace
    const { userDetails } = getUserDetails(userId);
    const userEmail = userDetails?.email || "unknown user";
    throw new Error(
      `User ${userEmail} is logged in but no workspace is selected.\n` +
        "Please run 'si login' again to select a workspace.",
    );
  }

  const { workspaceDetails, token } = getWorkspaceDetails(userId, workspaceId);

  if (!workspaceDetails) {
    throw new Error(
      `Workspace configuration not found for workspace ${workspaceId}.\n` +
        "The workspace may have been deleted or you may no longer have access.\n" +
        "Please run 'si login' again to select a different workspace.",
    );
  }

  if (!token) {
    throw new Error(
      `No API token found for workspace ${
        workspaceDetails.displayName || workspaceId
      }.\n` + "Please run 'si login' again to regenerate your workspace token.",
    );
  }

  return {
    authApiUrl,
    authApiToken,
    baseUrl: workspaceDetails.baseUrl,
    apiToken: token,
  };
}

/**
 * Checks if stored configuration is available.
 *
 * @returns true if we have a current user, workspace, and their associated tokens
 */
export function hasStoredConfig(): boolean {
  try {
    const userId = getCurrentUser();
    if (!userId) {
      return false;
    }

    const workspaceId = getCurrentWorkspace();
    if (!workspaceId) {
      return false;
    }

    const { workspaceDetails, token } = getWorkspaceDetails(
      userId,
      workspaceId,
    );
    return !!(workspaceDetails && token);
  } catch {
    return false;
  }
}

/**
 * Logs out by clearing the current user and workspace settings.
 * This removes the stored authentication state, requiring a new login.
 */
export function logout(): void {
  // Get the current user and workspace IDs before removing the files
  const userId = getCurrentUser();
  const workspaceId = getCurrentWorkspace();

  // Remove the workspace folder if it exists
  if (userId && workspaceId) {
    try {
      const workspacePath = join(authPath(), userId, workspaceId);
      Deno.removeSync(workspacePath, { recursive: true });
    } catch (error) {
      if (!(error instanceof Deno.errors.NotFound)) {
        console.warn(`Failed to remove workspace folder: ${error}`);
      }
    }
  }

  // Remove the user folder if it exists
  if (userId) {
    try {
      const userPath = join(authPath(), userId);
      Deno.removeSync(userPath, { recursive: true });
    } catch (error) {
      if (!(error instanceof Deno.errors.NotFound)) {
        console.warn(`Failed to remove user folder: ${error}`);
      }
    }
  }

  try {
    // Remove current workspace file
    Deno.removeSync(currentWorkspacePath());
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw error;
    }
  }

  try {
    // Remove current user file
    Deno.removeSync(currentUserPath());
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw error;
    }
  }
}

export const getApiUrlForInstance = (instanceUrl: string): string => {
  const url = new URL(instanceUrl);
  if (url.hostname == "localhost" || url.hostname == "127.0.0.1") {
    return `${url.protocol}//${url.hostname}:${5380}`;
  } else if (url.hostname == "app.systeminit.com") {
    return `${url.protocol}//api.systeminit.com`;
  } else if (url.hostname == "tools.systeminit.com") {
    return `${url.protocol}//api.tools.systeminit.com`;
  } else {
    return `${url.protocol}//api.${url.host}`;
  }
};
