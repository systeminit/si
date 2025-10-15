/**
 * Change Sets API Client
 *
 * Client for interacting with Change Sets endpoints
 */

import { ApiResponse, LuminorkClient } from '../client.ts';

// Type definitions for Change Sets API
export interface ChangeSetView {
  id: string;
  name: string;
  description?: string;
  status: string;
  created_at: string;
  last_updated_at: string;
}

export interface CreateChangeSetRequest {
  changeSetName: string;
}

export interface ChangeSetData {
  id: string;
  name: string;
  status: string;
  isHead: boolean;
}

export interface CreateChangeSetResponse {
  changeSet: ChangeSetData;
}

export interface ListChangeSetsResponse {
  changeSets: ChangeSetData[];
}

export interface GetChangeSetResponse {
  changeSet: ChangeSetData;
}

export interface ForceApplyChangeSetResponse {
  success: boolean;
  message?: string;
}

export interface RequestApprovalChangeSetResponse {
  success: boolean;
  message?: string;
}

export interface MergeStatusResponse {
  status: string;
  actions: Array<{
    id: string;
    component_id: string;
    name: string;
    status: string;
  }>;
}

export interface PurgeOpenChangeSetsResponse {
  success: boolean;
  purged_count: number;
}

/**
 * Change Sets API client for the Luminork API
 */
export class ChangeSetsApi {
  private client: LuminorkClient;

  constructor(client: LuminorkClient) {
    this.client = client;
  }

  /**
   * Build the full path with workspace ID
   */
  private buildPath(workspaceId: string, path: string): string {
    return `/v1/w/${workspaceId}/change-sets${path}`;
  }

  /**
   * Create a new change set
   */
  async createChangeSet(
    workspaceId: string,
    data: CreateChangeSetRequest,
  ): Promise<ApiResponse<CreateChangeSetResponse>> {
    return this.client.post<CreateChangeSetResponse>(
      this.buildPath(workspaceId, ''),
      data,
    );
  }

  /**
   * List all change sets in a workspace
   */
  async listChangeSets(
    workspaceId: string,
  ): Promise<ApiResponse<ListChangeSetsResponse>> {
    return this.client.get<ListChangeSetsResponse>(
      this.buildPath(workspaceId, ''),
    );
  }

  /**
   * Get a specific change set
   */
  async getChangeSet(
    workspaceId: string,
    changeSetId: string,
  ): Promise<ApiResponse<GetChangeSetResponse>> {
    return this.client.get<GetChangeSetResponse>(
      this.buildPath(workspaceId, `/${changeSetId}`),
    );
  }

  /**
   * Delete/abandon a change set
   */
  async deleteChangeSet(
    workspaceId: string,
    changeSetId: string,
  ): Promise<ApiResponse<{ success: boolean }>> {
    return this.client.delete(this.buildPath(workspaceId, `/${changeSetId}`));
  }

  /**
   * Force apply a change set
   */
  async forceApplyChangeSet(
    workspaceId: string,
    changeSetId: string,
  ): Promise<ApiResponse<ForceApplyChangeSetResponse>> {
    return this.client.post<ForceApplyChangeSetResponse>(
      this.buildPath(workspaceId, `/${changeSetId}/force_apply`),
      undefined,
    );
  }

  /**
   * Request approval for a change set
   */
  async requestApproval(
    workspaceId: string,
    changeSetId: string,
  ): Promise<ApiResponse<RequestApprovalChangeSetResponse>> {
    return this.client.post<RequestApprovalChangeSetResponse>(
      this.buildPath(workspaceId, `/${changeSetId}/request_approval`),
      undefined,
    );
  }

  /**
   * Get the merge status of a change set
   */
  async getMergeStatus(
    workspaceId: string,
    changeSetId: string,
  ): Promise<ApiResponse<MergeStatusResponse>> {
    return this.client.get<MergeStatusResponse>(
      this.buildPath(workspaceId, `/${changeSetId}/merge_status`),
    );
  }

  /**
   * Purge all open change sets in a workspace
   */
  async purgeOpenChangeSets(
    workspaceId: string,
  ): Promise<ApiResponse<PurgeOpenChangeSetsResponse>> {
    return this.client.post<PurgeOpenChangeSetsResponse>(
      this.buildPath(workspaceId, '/purge_open'),
      undefined,
    );
  }
}
