import { Module } from "vuex";
import _ from "lodash";

import { Workspace } from "@/api/sdf/model/workspace";
import { RootStore } from "@/store";

export interface WorkspaceStore {
  current: null | Workspace;
}

export const workspace: Module<WorkspaceStore, RootStore> = {
  namespaced: true,
  state: {
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
    current(state, payload: Workspace | null) {
      state.current = payload;
    },
  },
  actions: {
    async default({ commit, state }): Promise<Workspace> {
      const items = await Workspace.find("name", "default");
      if (items.length) {
        commit("current", items[0]);
      } else {
        throw new Error("cannot find default workspace");
      }
      // @ts-ignore - we know you think it could be null, but it can't!
      return state.current;
    },
    async fromDb({ commit, state }, payload: Workspace): Promise<void> {
      if (state.current?.id === payload.id) {
        commit("current", payload);
      }
    },
  },
};
