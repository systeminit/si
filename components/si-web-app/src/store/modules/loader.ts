import { Module } from "vuex";
import _ from "lodash";

import { sdf } from "@/api/sdf";
import { RootStore } from "@/store";

export interface LoaderStore {
  loading: boolean;
  loaded: boolean;
  nextUp: null | any;
}

export const loader: Module<LoaderStore, RootStore> = {
  namespaced: true,
  state: {
    loading: false,
    loaded: false,
    nextUp: null,
  },
  mutations: {
    loading(state, payload: boolean) {
      state.loading = payload;
    },
    loaded(state, payload: boolean) {
      state.loaded = payload;
    },
    nextUp(state, payload: any) {
      state.nextUp = payload;
    },
    clear(state) {
      state.loading = false;
      state.loaded = false;
      state.nextUp = null;
    },
  },
  actions: {
    async load({ state, commit, dispatch, rootState }): Promise<void> {
      if (!state.loaded) {
        commit("loading", true);
        await sdf.startUpdate();
        await dispatch("billingAccount/forUser", {}, { root: true });
        await dispatch("organization/default", {}, { root: true });
        await dispatch("workspace/default", {}, { root: true });
        await dispatch("system/default", {}, { root: true });
        let workspaceId = rootState.workspace.current?.id;
        if (sdf.update && workspaceId) {
          await sdf.update.loadData(workspaceId);
        }
        commit("loading", false);
      }
    },
    async logout({ commit }) {
      commit("loaded", false);
    },
    async clear({ commit }) {
      commit("clear");
    },
  },
};
