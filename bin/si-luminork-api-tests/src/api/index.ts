/**
 * Luminork API Client
 *
 * Main entry point for the Luminork API client
 */

import { ApiClientConfig, ApiResponse, LuminorkClient } from '../client.ts';
import { ChangeSetsApi } from './change-sets.ts';
import { ComponentsApi } from './components.ts';
import { SchemasApi } from './schemas.ts';

/**
 * Token information in whoami response
 */
export interface TokenInfo {
  iat: number;
  exp: number;
  sub: string;
  jti: string;
  version: string;
  userId: string;
  workspaceId: string;
  role: string;
}

/**
 * Interface for the whoami response
 */
export interface WhoamiResponse {
  userId: string;
  userEmail: string;
  workspaceId: string;
  token: TokenInfo;
}

/**
 * Interface for the system status response
 */
export interface SystemStatusResponse {
  'What is this?': string;
  'API Documentation': string;
}

/**
 * Main API client for interacting with the Luminork API server
 */
export class LuminorkApi {
  private client: LuminorkClient;

  // API clients for specific resource types
  public changeSets: ChangeSetsApi;
  public components: ComponentsApi;
  public schemas: SchemasApi;

  constructor(config: Partial<ApiClientConfig> = {}) {
    this.client = new LuminorkClient(config);

    // Initialize API resources
    this.changeSets = new ChangeSetsApi(this.client);
    this.components = new ComponentsApi(this.client);
    this.schemas = new SchemasApi(this.client);
  }

  /**
   * Set the authentication token for API requests
   */
  setAuthToken(token: string): void {
    this.client.setAuthToken(token);
  }

  /**
   * Update client configuration
   */
  updateConfig(config: Partial<ApiClientConfig>): void {
    this.client.updateConfig(config);
  }

  /**
   * Get system status
   */
  async getSystemStatus(): Promise<ApiResponse<SystemStatusResponse>> {
    return this.client.getSystemStatus();
  }

  /**
   * Get current user information
   */
  async whoami(): Promise<ApiResponse<WhoamiResponse>> {
    return this.client.whoami();
  }
}
