import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { watch } from "vue";
import { ApiRequest } from "@si/vue-lib/pinia";
import { UserId, useAuthStore, User } from "./auth.store";
import { ISODateString } from "./shared-types";

export type WorkspaceId = string;
export type AuthTokenId = string;

// TODO: do we want to share this type with the backend?
export type Workspace = {
  id: WorkspaceId;
  instanceEnvType: "LOCAL" | "PRIVATE" | "SI";
  instanceUrl: string;
  displayName: string;
  description?: string;
  slug: string;
  creatorUserId: UserId;
  creatorUser: {
    firstName?: string;
    lastName?: string;
  };
  createdAt: ISODateString;
  role: string;
  invitedAt: Date;
  isDefault: boolean;
  isFavourite: boolean;
  isHidden: boolean;
  quarantinedAt: Date;
  approvalStatus: boolean;
};

export type WorkspaceMember = {
  userId: UserId;
  nickname: string;
  email: string;
  role: string;
  signupAt: Date;
};

export type WorkspaceLookup = {
  firstName?: string | null;
  lastName?: string | null;
  email?: string | null;
  displayName: string;
  instanceUrl: string | null;
};

export type RumReportEntry = {
  id: string;
  email: string;
  nickname: string;
  signupAt: string;
  maxRum: number;
};

export const useWorkspacesStore = defineStore("workspaces", {
  state: () => ({
    workspacesById: {} as Record<WorkspaceId, Workspace>,
    selectedWorkspaceMembersById: {} as Record<UserId, WorkspaceMember>,
    workspaceForOwner: null as WorkspaceLookup | null,
    rumReport: [] as RumReportEntry[],
  }),
  getters: {
    workspaces: (state) => _.values(state.workspacesById),
    selectedWorkspaceMembers: (state) =>
      _.values(state.selectedWorkspaceMembersById),
    // grabbing the oldest workspace you created and assuming that it's your "default"
    defaultWorkspace: (state) => {
      const authStore = useAuthStore();

      // Let's first check for a defaultWorkspace
      const defaultWorkspace = _.head(
        _.filter(
          _.values(state.workspacesById),
          (w) => w.isDefault && w.creatorUserId === authStore.user?.id,
        ),
      );
      if (defaultWorkspace) return defaultWorkspace;

      // There's no direct defaultWorkspace so get the first created production workspace for that user
      const firstProductionWorkspace = _.head(
        _.sortBy(
          _.filter(
            _.values(state.workspacesById),
            (w) =>
              w.creatorUserId === authStore.user?.id &&
              w.instanceEnvType === "SI",
          ),
          (w) => w.createdAt,
        ),
      );
      if (firstProductionWorkspace) return firstProductionWorkspace;

      // This user has no production workspaces so we should not
      // redirect them anywhere but the workspaces page... for now!
      return null;
    },
  },
  actions: {
    // Reloads workspaces immediately and reactively when the user logs in.
    refreshWorkspaces() {
      if (!import.meta.env.SSR) {
        const authStore = useAuthStore();
        watch(
          () => authStore.userIsLoggedIn,
          async (userIsLoggedIn) => {
            if (userIsLoggedIn) await this.LOAD_WORKSPACES();
          },
          { immediate: true },
        );
      }
      return this.getRequestStatus("LOAD_WORKSPACES");
    },
    async LOAD_WORKSPACES() {
      return new ApiRequest<Workspace[]>({
        url: "/workspaces",
        onSuccess: (response) => {
          this.workspacesById = _.keyBy(response, (w) => w.id);
        },
      });
    },
    async LOAD_WORKSPACE_MEMBERS(workspaceId: WorkspaceId) {
      return new ApiRequest<WorkspaceMember[]>({
        url: `/workspace/${workspaceId}/members`,
        onSuccess: (response) => {
          this.selectedWorkspaceMembersById = _.keyBy(
            response,
            (u) => u.userId,
          );
        },
      });
    },
    async GET_WORKSPACE_OWNER(workspaceId: WorkspaceId) {
      return new ApiRequest<WorkspaceLookup>({
        url: `/workspaces/admin-lookup/${workspaceId}`,
        method: "get",
        onSuccess: (response) => {
          this.workspaceForOwner = response;
        },
      });
    },
    async CREATE_WORKSPACE(workspace: Partial<Workspace>) {
      return new ApiRequest<{
        workspaces: Workspace[];
        newWorkspaceId: string;
      }>({
        method: "post",
        url: "/workspaces/new",
        params: workspace,
        onSuccess: (response) => {
          this.workspacesById = _.keyBy(response.workspaces, (w) => w.id);
        },
      });
    },
    async SETUP_PRODUCTION_WORKSPACE(userEmail: string) {
      return new ApiRequest<{
        newWorkspace: Workspace;
      }>({
        method: "post",
        url: "/workspaces/setup-production-workspace",
        params: {
          userEmail,
        },
      });
    },

    async SETUP_PRODUCTION_WORKSPACE_BY_USER_ID(userId: string) {
      return new ApiRequest<{
        newWorkspace: Workspace;
      }>({
        method: "post",
        url: "/workspaces/setup-production-workspace-by-userid",
        params: {
          userId,
        },
      });
    },

    async DELETE_WORKSPACE(workspaceId: WorkspaceId) {
      return new ApiRequest({
        method: "delete",
        url: `/workspaces/${workspaceId}`,
        onSuccess: (response) => {
          // TODO
        },
      });
    },

    async EDIT_WORKSPACE(workspace: Partial<Workspace>) {
      return new ApiRequest<Workspace[]>({
        method: "patch",
        url: `/workspaces/${workspace.id}`,
        params: workspace,
        onSuccess: (response) => {
          this.workspacesById = _.keyBy(response, (w) => w.id);
        },
      });
    },

    async SET_WORKSPACE_QUARANTINE(
      workspaceId: string,
      isQuarantined: boolean,
    ) {
      return new ApiRequest<{ user: User }>({
        method: "patch",
        url: `/workspaces/${workspaceId}/quarantine`,
        params: {
          isQuarantined,
        },
      });
    },
    async SET_FAVOURITE(workspaceId: string, isFavourite: boolean) {
      return new ApiRequest<{ user: User }>({
        method: "patch",
        url: `/workspaces/${workspaceId}/favourite`,
        params: {
          isFavourite,
        },
      });
    },
    async SET_HIDDEN(workspaceId: string, isHidden: boolean) {
      return new ApiRequest<{ user: User }>({
        method: "patch",
        url: `/workspaces/${workspaceId}/setHidden`,
        params: {
          isHidden,
        },
      });
    },
    async SET_DEFAULT_WORKSPACE(workspaceId: string) {
      return new ApiRequest<{ user: User }>({
        method: "patch",
        url: `/workspaces/${workspaceId}/setDefault`,
      });
    },
    async CHANGE_WORKSPACE_APPROVAL_STATUS(
      workspaceId: string,
      approvalsEnabled: boolean,
    ) {
      return new ApiRequest<{ user: User }>({
        method: "patch",
        url: `/workspaces/${workspaceId}/approvalsEnabled`,
        params: {
          approvalsEnabled,
        },
      });
    },

    async INVITE_USER(
      userinfo: { email: string; role: string },
      workspaceId: WorkspaceId,
    ) {
      return new ApiRequest<WorkspaceMember>({
        method: "post",
        url: `/workspace/${workspaceId}/members`,
        params: userinfo,
        onSuccess: (response) => {
          // this.selectedWorkspaceMembersById[response.userId] = response;
          this.selectedWorkspaceMembersById = _.keyBy(
            response as never,
            (u) => u.userId,
          );
        },
      });
    },

    async CHANGE_MEMBERSHIP(
      workspaceId: WorkspaceId,
      userId: UserId,
      role: string,
    ) {
      return new ApiRequest<WorkspaceMember[]>({
        method: "post",
        url: `/workspace/${workspaceId}/membership`,
        params: {
          userId,
          role,
        },
        onSuccess: (response) => {
          this.selectedWorkspaceMembersById = _.keyBy(
            response as never,
            (u) => u.userId,
          );
        },
      });
    },

    async REMOVE_USER(email: string, workspaceId: WorkspaceId) {
      return new ApiRequest<WorkspaceMember[]>({
        method: "delete",
        url: `/workspace/${workspaceId}/members`,
        params: {
          email,
        },
        onSuccess: (response) => {
          this.selectedWorkspaceMembersById = _.keyBy(
            response,
            (u) => u.userId,
          );
        },
      });
    },

    async LEAVE_WORKSPACE(workspaceId: WorkspaceId) {
      return new ApiRequest<WorkspaceMember[]>({
        method: "delete",
        url: `/workspace/${workspaceId}/leave`,
        onSuccess: (response) => {
          // Remove the workspace from the local store since we're no longer a member
          delete this.workspacesById[workspaceId];
          // Clear the members list
          this.selectedWorkspaceMembersById = {};
        },
      });
    },
    async GET_RUM_REPORT(month?: string) {
      return new ApiRequest<RumReportEntry[]>({
        method: "get",
        url: "/rum-report",
        params: month ? { month } : undefined,
        onSuccess: (response) => {
          this.rumReport = response;
        },
      });
    },
  },
});
