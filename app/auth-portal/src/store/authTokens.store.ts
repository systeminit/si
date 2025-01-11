import { defineStore } from "pinia";
import { ApiRequest } from "@si/vue-lib/pinia";
import { WorkspaceId } from "./workspaces.store";

export type AuthTokenId = string;
export interface AuthToken {
  id: string;
  name: string | null;
  userId: string;
  workspaceId: string;
  createdAt: Date;
  expiresAt: Date | null;
  claims: unknown;
  lastUsedAt: Date | null;
  lastUsedIp: string | null;
}

export const useAuthTokensApi = defineStore("authTokens", {
  actions: {
    async FETCH_AUTH_TOKENS(workspaceId: WorkspaceId) {
      return new ApiRequest<{ authTokens: AuthToken[] }>({
        url: ["workspaces", { workspaceId }, "authTokens"],
        keyRequestStatusBy: workspaceId,
      });
    },

    async CREATE_AUTH_TOKEN(
      workspaceId: WorkspaceId,
      name?: string,
      expiration?: string,
    ) {
      return new ApiRequest<{ authToken: AuthToken; token: string }>({
        method: "post",
        url: ["workspaces", { workspaceId }, "authTokens"],
        params: { name, expiration },
        keyRequestStatusBy: workspaceId,
      });
    },

    async FETCH_AUTH_TOKEN(workspaceId: WorkspaceId, tokenId: AuthTokenId) {
      return new ApiRequest<{ authToken: AuthToken }>({
        url: ["workspaces", { workspaceId }, "authTokens", { tokenId }],
        keyRequestStatusBy: [workspaceId, tokenId],
      });
    },

    async RENAME_AUTH_TOKEN(
      workspaceId: WorkspaceId,
      tokenId: AuthTokenId,
      name: string | null,
    ) {
      return new ApiRequest<void>({
        method: "put",
        url: ["workspaces", { workspaceId }, "authTokens", { tokenId }],
        params: { name },
        keyRequestStatusBy: [workspaceId, tokenId],
      });
    },

    async REVOKE_AUTH_TOKEN(workspaceId: WorkspaceId, tokenId: AuthTokenId) {
      return new ApiRequest<void>({
        method: "delete",
        url: ["workspaces", { workspaceId }, "authTokens", { tokenId }],
        keyRequestStatusBy: [workspaceId, tokenId],
      });
    },
  },
});
