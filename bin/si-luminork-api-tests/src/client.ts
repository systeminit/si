/**
 * Luminork API Client
 *
 * A client for interacting with the Luminork API service.
 * Provides methods for authentication and interacting with various endpoints.
 */

// Default configuration values
const DEFAULT_CONFIG = {
  baseUrl: 'http://localhost:5380',
  timeout: 30000,
};

// Types for request/response handling
export interface ApiClientConfig {
  baseUrl: string;
  timeout: number;
  authToken?: string;
}

export interface RequestOptions {
  method: string;
  headers?: Record<string, string>;
  body?: unknown;
  timeout?: number;
}

export interface ApiResponse<T> {
  status: number;
  statusText: string;
  data: T;
  headers: Headers;
}

export class ApiError extends Error {
  status: number;
  statusText: string;
  data: unknown;

  constructor(
    message: string,
    status: number,
    statusText: string,
    data: unknown,
  ) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.statusText = statusText;
    this.data = data;
  }
}

export class ConfigError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ConfigError';
  }
}

/**
 * LuminorkClient provides a simple client for interacting with the Luminork API
 */
export class LuminorkClient {
  private config: ApiClientConfig;

  constructor(config: Partial<ApiClientConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Set the authentication token for API requests
   */
  setAuthToken(token: string): void {
    this.config.authToken = token;
  }

  /**
   * Update client configuration
   */
  updateConfig(config: Partial<ApiClientConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get the base URL
   */
  getBaseUrl(): string {
    return this.config.baseUrl;
  }

  /**
   * Perform a request to the API
   */
  async request<T = unknown>(
    path: string,
    options: RequestOptions,
  ): Promise<ApiResponse<T>> {
    const url = new URL(path, this.config.baseUrl);

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      Accept: 'application/json',
      ...options.headers,
    };

    // Add auth token if available
    if (this.config.authToken) {
      headers['Authorization'] = `Bearer ${this.config.authToken}`;
    }

    // Prepare request options
    const requestInit: RequestInit = {
      method: options.method,
      headers,
    };

    // Add body for non-GET requests
    if (options.method !== 'GET' && options.body) {
      requestInit.body = JSON.stringify(options.body);
    }

    // Handle timeout
    const timeout = options.timeout || this.config.timeout;

    try {
      // Create abort controller for timeout handling
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);
      requestInit.signal = controller.signal;

      const response = await fetch(url.toString(), requestInit);
      clearTimeout(timeoutId);

      // Parse response
      let data: T;
      const contentType = response.headers.get('content-type');

      if (contentType && contentType.includes('application/json')) {
        data = (await response.json()) as T;
      } else {
        data = (await response.text()) as unknown as T;
      }

      if (!response.ok) {
        throw new ApiError(
          `API request failed: ${response.status} ${response.statusText}`,
          response.status,
          response.statusText,
          data,
        );
      }

      return {
        status: response.status,
        statusText: response.statusText,
        data,
        headers: response.headers,
      };
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }

      if (error instanceof DOMException && error.name === 'AbortError') {
        throw new ApiError(
          `Request timeout after ${timeout}ms`,
          408,
          'Request Timeout',
          { error: 'timeout' },
        );
      }

      throw new ApiError(
        `API request failed: ${error instanceof Error ? error.message : String(error)}`,
        500,
        'Internal Error',
        { error: String(error) },
      );
    }
  }

  /**
   * GET request helper
   */
  async get<T = unknown>(
    path: string,
    options: Omit<RequestOptions, 'method' | 'body'> = {},
  ): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'GET' });
  }

  /**
   * POST request helper
   */
  async post<T = unknown>(
    path: string,
    body: unknown = undefined,
    options: Omit<RequestOptions, 'method' | 'body'> = {},
  ): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'POST', body });
  }

  /**
   * PUT request helper
   */
  async put<T = unknown>(
    path: string,
    body: unknown = undefined,
    options: Omit<RequestOptions, 'method' | 'body'> = {},
  ): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'PUT', body });
  }

  /**
   * DELETE request helper
   */
  async delete<T = unknown>(
    path: string,
    options: Omit<RequestOptions, 'method' | 'body'> = {},
  ): Promise<ApiResponse<T>> {
    return this.request<T>(path, { ...options, method: 'DELETE' });
  }

  /**
   * Check system status
   */
  async getSystemStatus(): Promise<
    ApiResponse<{ 'What is this?': string; 'API Documentation': string }>
  > {
    return this.get<{ 'What is this?': string; 'API Documentation': string }>('/');
  }

  /**
   * Get current user information
   */
  async whoami(): Promise<
    ApiResponse<{
      userId: string;
      userEmail: string;
      workspaceId: string;
      token: {
        iat: number;
        exp: number;
        sub: string;
        jti: string;
        version: string;
        userId: string;
        workspaceId: string;
        role: string;
      };
    }>
  > {
    return this.get<{
      userId: string;
      userEmail: string;
      workspaceId: string;
      token: {
        iat: number;
        exp: number;
        sub: string;
        jti: string;
        version: string;
        userId: string;
        workspaceId: string;
        role: string;
      };
    }>('/whoami');
  }
}
