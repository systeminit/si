import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { ApiRequest } from "@si/vue-lib/pinia";
import { UserId } from "./auth.store";
import { ISODateString } from "./shared-types";

export type WorkspaceId = string;

// TODO: do we want to share this type with the backend?
export type Workspace = {
  id: WorkspaceId;
  instanceType: "local" | "private" | "si_sass"; // only local used for now...
  instanceUrl: string;
  displayName: string;
  slug: string;
  createdByUserId: UserId;
  createdAt: ISODateString;
  role: string;
};

export type WorkspaceMember = {
  userId: UserId;
  nickname: string;
  email: string;
  role: string;
  signupAt: Date;
};

export const useWorkspacesStore = defineStore("workspaces", {
  state: () => ({
    workspacesById: {} as Record<WorkspaceId, Workspace>,
    selectedWorkspaceMembersById: {} as Record<UserId, WorkspaceMember>,
  }),
  getters: {
    workspaces: (state) => _.values(state.workspacesById),
    selectedWorkspaceMembers: (state) =>
      _.values(state.selectedWorkspaceMembersById),
    // when we have multiple workspaces, we'll want to track which one was the default...
    defaultWorkspace: (state) => _.values(state.workspacesById)[0],
  },
  actions: {
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
  },
});
