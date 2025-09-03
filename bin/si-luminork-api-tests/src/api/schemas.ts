/**
 * Schemas API Client
 *
 * Client for interacting with Schemas endpoints
 */

import { LuminorkClient, ApiResponse } from "../client.ts";

// Type definitions for Schemas API
export interface SchemaView {
  schemaId: string;
  schemaName: string;
  category?: string;
  installed: boolean;
}

export interface SchemaVariantView {
  id: string;
  name: string;
  description?: string;
  version: string;
  schema_id: string;
}

export interface ListSchemasResponse {
  schemas: SchemaView[];
  nextCursor?: string;
}

export interface GetSchemaResponse {
  id: string;
  name: string;
  category: string;
  display_name: string;
  created_at: string;
  updated_at: string;
  variants: SchemaVariantView[];
}

export interface FindSchemaParams {
  name?: string;
  id?: string;
}

export interface GetSchemaVariantResponse {
  id: string;
  name: string;
  schema_id: string;
  version: string;
  created_at: string;
  updated_at: string;
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
        throw new Error("No HEAD change set found");
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
        throw new Error("No HEAD change set found");
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
      url.searchParams.append("schema", params.name);
    }

    if (params.id) {
      url.searchParams.append("schemaId", params.id);
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

  async getDefaultSchemaVariant(
    workspaceId: string,
    changeSetId: string,
    schemaId: string,
  ): Promise<ApiResponse<GetSchemaVariantResponse>> {
    return this.client.get<GetSchemaVariantResponse>(
      `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}/variant/default`,
    );
  }
}
