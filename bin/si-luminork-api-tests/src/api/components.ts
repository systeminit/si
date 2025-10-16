/**
 * Components API Client
 *
 * Client for interacting with Components endpoints
 */

import { ApiResponse, LuminorkClient } from '../client.ts';

// Type definitions for Components API
export interface ComponentView {
  id: string;
  name: string;
  resourceId: string;
  toDelete: boolean;
  canBeUpgraded: boolean;
  connections: Array<unknown>;
  views: Array<{
    id: string;
    name: string;
    isDefault: boolean;
  }>;
  sockets?: Array<unknown>;
  domainProps?: Array<unknown>;
  resourceProps?: Array<unknown>;
}

// Attribute value types that can be used in component attributes
export type AttributeValue =
  | { $source: null | { component: string; path: string; func?: string } }
  | string
  | unknown[]
  | boolean
  | Record<string, unknown>
  | number;

export interface CreateComponentRequest {
  name: string;
  schemaName: string;
  resourceId?: string;
  viewName?: string;
  managedBy?: {
    component?: string;
  };
  attributes?: Record<string, AttributeValue>;
  useWorkingCopy?: boolean;
}

export interface UpdateComponentRequest {
  name?: string;
  props?: Record<string, unknown>;
}

export interface CreateComponentResponse {
  component: {
    id: string;
    name: string;
    resourceId: string;
    toDelete: boolean;
    canBeUpgraded: boolean;
    connections: Array<unknown>;
    views: Array<{
      id: string;
      name: string;
      isDefault: boolean;
    }>;
    sockets?: Array<unknown>;
    domainProps?: Array<unknown>;
    resourceProps?: Array<unknown>;
  };
}

export interface UpdateComponentResponse {
  component: {
    id: string;
    name: string;
    resourceId: string;
    toDelete: boolean;
    canBeUpgraded: boolean;
    connections: Array<unknown>;
    views: Array<{
      id: string;
      name: string;
      isDefault: boolean;
    }>;
    sockets?: Array<unknown>;
    domainProps?: Array<unknown>;
    resourceProps?: Array<unknown>;
  };
}

export interface ComponentDetails {
  componentId: string;
  name: string;
  schemaName: string;
}

export interface ListComponentsResponse {
  componentDetails: ComponentDetails[];
  nextCursor: string | null;
}

export interface GetComponentResponse {
  component: {
    id: string;
    name: string;
    resourceId: string;
    toDelete: boolean;
    canBeUpgraded: boolean;
    connections: Array<unknown>;
    views: Array<{
      id: string;
      name: string;
      isDefault: boolean;
    }>;
    sockets?: Array<unknown>;
    domainProps?: Array<unknown>;
    resourceProps?: Array<unknown>;
    action_functions?: Array<unknown>;
    management_functions?: Array<unknown>;
  };
}

export interface FindComponentParams {
  name?: string;
  schema_id?: string;
}

export interface ExecuteManagementFunctionRequest {
  function_id: string;
  arguments?: Record<string, unknown>;
}

export interface ExecuteManagementFunctionResponse {
  success: boolean;
  result?: unknown;
}

export interface AddActionRequest {
  action_id: string;
  arguments?: Record<string, unknown>;
}

export interface AddActionResponse {
  action_id: string;
  component_id: string;
}

/**
 * Components API client for the Luminork API
 */
export class ComponentsApi {
  private client: LuminorkClient;

  constructor(client: LuminorkClient) {
    this.client = client;
  }

  /**
   * Build the full path with workspace and changeset IDs
   */
  private buildPath(
    workspaceId: string,
    changeSetId: string,
    path: string,
  ): string {
    return `/v1/w/${workspaceId}/change-sets/${changeSetId}/components${path}`;
  }

  /**
   * Create a new component
   */
  async createComponent(
    workspaceId: string,
    changeSetId: string,
    data: CreateComponentRequest,
  ): Promise<ApiResponse<CreateComponentResponse>> {
    return this.client.post<CreateComponentResponse>(
      this.buildPath(workspaceId, changeSetId, ''),
      data,
    );
  }

  /**
   * List all components in a change set
   */
  async listComponents(
    workspaceId: string,
    changeSetId: string,
  ): Promise<ApiResponse<ListComponentsResponse>> {
    return this.client.get<ListComponentsResponse>(
      this.buildPath(workspaceId, changeSetId, ''),
    );
  }

  /**
   * Find components by name or schema
   */
  async findComponent(
    workspaceId: string,
    changeSetId: string,
    params: FindComponentParams,
  ): Promise<ApiResponse<GetComponentResponse>> {
    const url = new URL(
      this.buildPath(workspaceId, changeSetId, '/find'),
      this.client.getBaseUrl(),
    );

    // Add search parameters
    if (params.name) {
      url.searchParams.append('component', params.name);
    }

    if (params.schema_id) {
      url.searchParams.append('schema_id', params.schema_id);
    }

    return this.client.get<GetComponentResponse>(url.toString());
  }

  /**
   * Get a specific component
   */
  async getComponent(
    workspaceId: string,
    changeSetId: string,
    componentId: string,
  ): Promise<ApiResponse<GetComponentResponse>> {
    return this.client.get<GetComponentResponse>(
      this.buildPath(workspaceId, changeSetId, `/${componentId}`),
    );
  }

  /**
   * Update a component
   */
  async updateComponent(
    workspaceId: string,
    changeSetId: string,
    componentId: string,
    data: UpdateComponentRequest,
  ): Promise<ApiResponse<UpdateComponentResponse>> {
    return this.client.put<UpdateComponentResponse>(
      this.buildPath(workspaceId, changeSetId, `/${componentId}`),
      data,
    );
  }

  /**
   * Delete a component
   */
  async deleteComponent(
    workspaceId: string,
    changeSetId: string,
    componentId: string,
  ): Promise<ApiResponse<{ success: boolean }>> {
    return this.client.delete(
      this.buildPath(workspaceId, changeSetId, `/${componentId}`),
    );
  }

  /**
   * Execute a management function on a component
   */
  async executeManagementFunction(
    workspaceId: string,
    changeSetId: string,
    componentId: string,
    data: ExecuteManagementFunctionRequest,
  ): Promise<ApiResponse<ExecuteManagementFunctionResponse>> {
    return this.client.post<ExecuteManagementFunctionResponse>(
      this.buildPath(
        workspaceId,
        changeSetId,
        `/${componentId}/execute-management-function`,
      ),
      data,
    );
  }

  /**
   * Add an action to a component
   */
  async addAction(
    workspaceId: string,
    changeSetId: string,
    componentId: string,
    data: AddActionRequest,
  ): Promise<ApiResponse<AddActionResponse>> {
    return this.client.post<AddActionResponse>(
      this.buildPath(workspaceId, changeSetId, `/${componentId}/action`),
      data,
    );
  }
}
