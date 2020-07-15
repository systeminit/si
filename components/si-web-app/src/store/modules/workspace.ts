import { Module } from "vuex";
import _ from "lodash";

import { Workspace } from "@/graphql-types";
import { RootStore } from "@/store";
import { graphqlQueryListAll } from "@/api/apollo";

export interface WorkspaceStore {
  workspaces: Workspace[];
  current: null | Workspace;
}

interface AddMutation {
  workspaces: Workspace[];
}

export const workspace: Module<WorkspaceStore, RootStore> = {
  namespaced: true,
  state: {
    workspaces: [],
    current: null,
  },
  getters: {
    current(state): Workspace {
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current workspace; it is not set!");
      }
    },
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.workspaces = _.unionBy(payload.workspaces, state.workspaces, "id");
    },
    current(state, payload: Workspace) {
      state.current = payload;
    },
  },
  actions: {
    async load({ commit }): Promise<void> {
      const workspaces: Workspace[] = await graphqlQueryListAll({
        typeName: "workspace",
      });
      if (workspaces.length > 0) {
        commit("add", { workspaces });
        const defaultWorkspace = _.find(workspaces, ["name", "default"]);
        if (defaultWorkspace) {
          commit("current", defaultWorkspace);
        } else {
          commit("current", workspaces[0]);
        }
      }
    },
  },
};
