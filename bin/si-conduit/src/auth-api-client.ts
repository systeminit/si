export interface WorkspaceDetails {
  instanceUrl: string;
  displayName: string;
  id: string;
  instanceEnvType: string;
}

export class AuthApiClient {
  private readonly apiToken: string;
  private readonly workspaceId: string;

  constructor(apiToken: string, workspaceId: string) {
    this.apiToken = apiToken;
    this.workspaceId = workspaceId;
  }

  async getWorkspaceDetails(): Promise<WorkspaceDetails> {
    const response = await fetch(
      `https://auth-api.systeminit.com/workspaces/${this.workspaceId}`,
      {
        headers: {
          Authorization: `Bearer ${this.apiToken}`,
        },
      }
    );

    if (!response.ok) {
      throw new Error(`Failed to fetch workspace details: ${response.statusText}`);
    }

    const { instanceUrl, displayName, id, instanceEnvType } = await response.json();

    return {
      instanceUrl,
      displayName,
      id,
      instanceEnvType,
    };
  }
}
