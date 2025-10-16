/**
 * Schemas API Client
 *
 * Client for interacting with Schemas endpoints
 */

import { ApiResponse, LuminorkClient } from '../client.ts';

// Type definitions for Schemas API
export interface SchemaView {
  schemaId: string;
  schemaName: string;
  category?: string;
  installed: boolean;
}

export interface ListSchemasResponse {
  schemas: SchemaView[];
  nextCursor?: string;
}

export interface GetSchemaResponse {
  defaultVariantId: string;
  name: string;
  schemaId: string;
  variantIds: string[];
}

export interface FindSchemaParams {
  name?: string;
  id?: string;
}

export interface GetSchemaVariantResponse {
  assetFuncId: string;
  category: string;
  color: string;
  description: string;
  displayName: string;
  domainProps: null;
  installedFromUpstream: boolean;
  isDefaultVariant: boolean;
  isLocked: boolean;
  link: string;
  variantFuncs: {
    funcKind: {
      actionKind: string;
      kind: string;
    } | {
      managementFuncKind: string;
      kind: string;
    } | {
      funcKind: string;
      kind: string;
    };
    id: string;
  }[];
  variantId: string;
}

export interface UpdateSchemaVariantBody {
  category: string;
  code: string;
  color: string;
  description: string;
  link: string;
  name: string;
}

export interface UpdateSchemaVariantParams {
  schema_id: string;
  schema_variant_id: string;
  request_body: UpdateSchemaVariantBody;
}

export interface CreateSchemaBody {
  name: string;
  description?: string;
  link?: string;
  category?: string;
  color?: string;
  code: string;
}

export interface CreateSchemaResponse {
  defaultVariantId: string;
  name: string;
  schemaId: string;
  variantIds: string[];
}

export interface UnlockSchemaResponse {
  schemaId: string;
  unlockedVariantId: string;
  unlockedVariant: GetSchemaVariantResponse;
}

export interface SearchSchemasBody {
  category?: string;
}

export interface SearchSchemasResponse {
  schemas: SchemaView[];
}

export interface CreateActionFuncBody {
  name: string;
  displayName?: string;
  description?: string;
  kind: 'Create' | 'Destroy' | 'Manual' | 'Refresh' | 'Update';
  code: string;
}

export interface CreateCodegenFuncBody {
  name: string;
  displayName?: string;
  description?: string;
  code: string;
}

export interface CreateQualificationFuncBody {
  name: string;
  displayName?: string;
  description?: string;
  code: string;
}

export interface CreateAuthenticationFuncBody {
  name: string;
  displayName?: string;
  description?: string;
  code: string;
}

export interface CreateManagementFuncBody {
  name: string;
  displayName?: string;
  description?: string;
  kind: 'Import' | 'Discover';
  code: string;
}

export interface CreateFuncResponse {
  funcId: string;
}

/**
 * Schemas API client for the Luminork API
 */
export class SchemasApi {
  private client: LuminorkClient;

  constructor(client: LuminorkClient) {
    this.client = client;
  }

  /**
   * List all schemas in a workspace
   */
  async listSchemas(
    workspaceId: string,
    changeSetId?: string,
  ): Promise<ApiResponse<ListSchemasResponse>> {
    if (changeSetId) {
      return this.client.get<ListSchemasResponse>(
        `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas`,
      );
    } else {
      // Fallback to using the HEAD change set if no change set ID is provided
      const changeSetsResponse = await this.client.get<any>(
        `/v1/w/${workspaceId}/change-sets`,
      );
      const headChangeSet = changeSetsResponse.data.changeSets.find(
        (cs: any) => cs.isHead === true,
      );
      if (!headChangeSet) {
        throw new Error('No HEAD change set found');
      }
      return this.client.get<ListSchemasResponse>(
        `/v1/w/${workspaceId}/change-sets/${headChangeSet.id}/schemas`,
      );
    }
  }

  /**
   * Find schemas by name or category
   */
  async findSchema(
    workspaceId: string,
    params: FindSchemaParams,
    changeSetId?: string,
  ): Promise<ApiResponse<SchemaView>> {
    if (!changeSetId) {
      // Fallback to using the HEAD change set if no change set ID is provided
      const changeSetsResponse = await this.client.get<any>(
        `/v1/w/${workspaceId}/change-sets`,
      );
      const headChangeSet = changeSetsResponse.data.changeSets.find(
        (cs: any) => cs.isHead === true,
      );
      if (!headChangeSet) {
        throw new Error('No HEAD change set found');
      }
      changeSetId = headChangeSet.id;
    }

    // Use the standard listing endpoint with query parameters
    const url = new URL(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/find`,
      this.client.getBaseUrl(),
    );

    // Add search parameters
    if (params.name) {
      url.searchParams.append('schema', params.name);
    }

    if (params.id) {
      url.searchParams.append('schemaId', params.id);
    }

    return this.client.get<SchemaView>(url.toString());
  }

  /**
   * Get a specific schema with its variants
   */
  async getSchema(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
  ): Promise<ApiResponse<GetSchemaResponse>> {
    return this.client.get<GetSchemaResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}`,
    );
  }

  /**
   * Get a specific schema variant
   */
  async getSchemaVariant(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
    variantId: string,
  ): Promise<ApiResponse<GetSchemaVariantResponse>> {
    return this.client.get<GetSchemaVariantResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/variant/${variantId}`,
    );
  }

  /**
   * Get the default schema variant
   */
  async getDefaultSchemaVariant(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
  ): Promise<ApiResponse<GetSchemaVariantResponse>> {
    return this.client.get<GetSchemaVariantResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/variant/default`,
    );
  }

  /**
   * Update a specific schema variant and regenerate it
   */
  async updateSchemaVariant(
    workspaceId: string,
    changeSetId: string,
    params: UpdateSchemaVariantParams,
  ): Promise<ApiResponse<GetSchemaVariantResponse>> {
    return this.client.put<GetSchemaVariantResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${params.schema_id}/variant/${params.schema_variant_id}`,
      params.request_body,
    );
  }

  /**
   * Create a new schema with its default variant
   */
  async createSchema(
    workspaceId: string,
    changeSetId: string,
    body: CreateSchemaBody,
  ): Promise<ApiResponse<CreateSchemaResponse>> {
    return this.client.post<CreateSchemaResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas`,
      body,
    );
  }

  /**
   * Unlock a schema - creates an unlocked variant if one doesn't exist
   */
  async unlockSchema(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
  ): Promise<ApiResponse<UnlockSchemaResponse>> {
    return this.client.post<UnlockSchemaResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/unlock`,
      {},
    );
  }

  /**
   * Search for schemas with complex filters
   */
  async searchSchemas(
    workspaceId: string,
    changeSetId: string,
    body: SearchSchemasBody,
  ): Promise<ApiResponse<SearchSchemasResponse>> {
    return this.client.post<SearchSchemasResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/search`,
      body,
    );
  }

  /**
   * Create an action function and attach it to a schema variant
   */
  async createActionFunc(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
    variantId: string,
    body: CreateActionFuncBody,
  ): Promise<ApiResponse<CreateFuncResponse>> {
    return this.client.post<CreateFuncResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/variant/${variantId}/funcs/action`,
      body,
    );
  }

  /**
   * Create a codegen function and attach it to a schema variant
   */
  async createCodegenFunc(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
    variantId: string,
    body: CreateCodegenFuncBody,
  ): Promise<ApiResponse<CreateFuncResponse>> {
    return this.client.post<CreateFuncResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/variant/${variantId}/funcs/codegen`,
      body,
    );
  }

  /**
   * Create a qualification function and attach it to a schema variant
   */
  async createQualificationFunc(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
    variantId: string,
    body: CreateQualificationFuncBody,
  ): Promise<ApiResponse<CreateFuncResponse>> {
    return this.client.post<CreateFuncResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/variant/${variantId}/funcs/qualification`,
      body,
    );
  }

  /**
   * Create an authentication function and attach it to a schema variant
   */
  async createAuthenticationFunc(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
    variantId: string,
    body: CreateAuthenticationFuncBody,
  ): Promise<ApiResponse<CreateFuncResponse>> {
    return this.client.post<CreateFuncResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/variant/${variantId}/funcs/authentication`,
      body,
    );
  }

  /**
   * Create a management function and attach it to a schema variant
   */
  async createManagementFunc(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
    variantId: string,
    body: CreateManagementFuncBody,
  ): Promise<ApiResponse<CreateFuncResponse>> {
    return this.client.post<CreateFuncResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/variant/${variantId}/funcs/management`,
      body,
    );
  }
}
