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
  Active = "Active",
  Error = "Error", 
  Terminated = "Terminated",
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
      `/api/v2/workspaces/${workspaceId}/change-sets/${changeSetId}/remote-shell/create`,
      request
    );

    return response.data as RemoteShellSessionDetails;
  }
}