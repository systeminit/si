import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { ApiRequest } from "@si/vue-lib/pinia";
import { UserId } from "./auth.store";
import { ISODateString } from "./shared-types";

type WorkspaceId = string;

// TODO: do we want to share this type with the backend?
type Workspace = {
  id: WorkspaceId;
  instanceType: "local" | "private" | "si_sass"; // only local used for now...
  instanceUrl: string;
  displayName: string;
  slug: string;
  createdByUserId: UserId;
  createdAt: ISODateString;
};

export const useWorkspacesStore = defineStore("workspaces", {
  state: () => ({
    workspacesById: {} as Record<WorkspaceId, Workspace>,
  }),
  getters: {
    workspaces: (state) => _.values(state.workspacesById),
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
  },
});
