import { Module } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";

import {
  BillingAccount,
  IBillingAccountCreateRequest,
  IBillingAccountCreateReply,
} from "@/api/sdf/model/billingAccount";

export interface BillingAccountStore {
  current: null | BillingAccount;
}

export const billingAccount: Module<BillingAccountStore, RootStore> = {
  namespaced: true,
  state: {
    current: null,
  },
  mutations: {
    current(state, payload: BillingAccount) {
      state.current = payload;
    },
  },
  actions: {
    async fromDb({ commit, state }, payload: BillingAccount): Promise<void> {
      if (state.current?.id === payload.id) {
        commit("current", payload);
      }
    },
    async create(
      _,
      payload: IBillingAccountCreateRequest,
    ): Promise<IBillingAccountCreateReply> {
      let response = await BillingAccount.create(payload);
      return response;
    },
  },
};
