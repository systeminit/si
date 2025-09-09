import { sdfApiInstance } from "@/store/apis.web";

export interface CreateRemoteShellSessionRequest {
  image?: string;
  workingDir?: string;
  envVars?: Record<string, string>;
}

export interface RemoteShellConnectionInfo {
  natsSubject: string;
  stdinSubject: string;
  stdoutSubject: string;
  stderrSubject: string;
  controlSubject: string;
}

export enum RemoteShellStatus {
  Active = "active",
  Error = "error", 
  Terminated = "terminated",
}

export interface CreateRemoteShellSessionResponse {
  executionId: string;
  sessionId: string;
  containerId: string;
  connectionInfo: RemoteShellConnectionInfo;
  status: RemoteShellStatus;
  message?: string;
}

export interface RemoteShellSessionDetails {
  forcedChangeSetId: string;
  data: CreateRemoteShellSessionResponse;
}

export class RemoteShellApi {
  static async createSession(
    workspaceId: string,
    changeSetId: string,
    request: CreateRemoteShellSessionRequest = {}
  ): Promise<RemoteShellSessionDetails> {
    const response = await sdfApiInstance.post(
      `/v2/workspaces/${workspaceId}/change-sets/${changeSetId}/remote-shell/create`,
      request
    );

    // The backend returns a ForceChangeSetResponse where:
    // - The actual data is in response.data directly (CreateRemoteShellSessionResponse)
    // - The force_change_set_id is in the response headers
    return {
      forcedChangeSetId: response.headers.force_change_set_id || '',
      data: response.data as CreateRemoteShellSessionResponse,
    };
  }
}