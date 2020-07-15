import { Module } from "vuex";
import _ from "lodash";

import { graphqlQuery } from "@/api/apollo";
import { RootStore } from "@/store";

export interface LoaderStore {
  loading: boolean;
}

export const loader: Module<LoaderStore, RootStore> = {
  namespaced: true,
  state: {
    loading: false,
  },
  mutations: {
    loading(state, payload: boolean) {
      state.loading = payload;
    },
  },
  actions: {
    async load({ commit, dispatch, rootState }): Promise<void> {
      commit("loading", true);
      await dispatch("workspace/load", {}, { root: true });
      await dispatch(
        "billingAccount/get",
        {
          billingAccountId: rootState.user.auth.profile?.billingAccount?.id,
        },
        { root: true },
      );
      await dispatch("changeSet/load", {}, { root: true });
      await dispatch("entity/load", {}, { root: true });
      commit("loading", false);
    },
  },
};
