import { Module } from "vuex";
import _ from "lodash";

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
  },
  actions: {
    async load({ state, commit, dispatch, rootState }): Promise<void> {
      if (!state.loaded) {
        commit("loading", true);
        await dispatch("billingAccount/forUser", {}, { root: true });
        await dispatch("workspace/default", {}, { root: true });
        await dispatch("organization/default", {}, { root: true });
        commit("loading", false);
        commit("loaded", true);
      }
    },
  },
};
