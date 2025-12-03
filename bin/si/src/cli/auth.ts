export interface WorkspaceDetails {
  instanceUrl: string;
  displayName: string;
  id: string;
  instanceEnvType: string;
}

export class AuthApiClient {
  private readonly apiToken: string;
  private readonly workspaceId: string;
  private readonly authApiUrl: string;

  constructor(authApiUrl: string, apiToken: string, workspaceId: string) {
    this.authApiUrl = authApiUrl;
    this.apiToken = apiToken;
    this.workspaceId = workspaceId;
  }

  async getWorkspaceDetails(): Promise<WorkspaceDetails> {
    const response = await fetch(
      `${this.authApiUrl}/workspaces/${this.workspaceId}`,
      {
        headers: {
          Authorization: `Bearer ${this.apiToken}`,
        },
      },
    );

    if (!response.ok) {
      throw new Error(
        `Failed to fetch workspace details: ${response.statusText}`,
      );
    }

    const { instanceUrl, displayName, id, instanceEnvType } = await response
      .json();

    if (!instanceUrl || !displayName || !id || !instanceEnvType) {
      throw new Error(
        "Failed to fetch workspace details: missing required fields",
      );
    }

    return {
      instanceUrl,
      displayName,
      id,
      instanceEnvType,
    };
  }
}
