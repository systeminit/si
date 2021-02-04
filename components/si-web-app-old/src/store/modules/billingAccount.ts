import { Module } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";

import { IUser } from "@/api/sdf/model/user";
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
    async forUser({ commit, rootGetters }): Promise<void> {
      let user: IUser = rootGetters["user/current"];
      let billingAccount = await BillingAccount.get({
        id: user.siStorable.billingAccountId,
      });
      commit("current", billingAccount);
    },
    async fromDb({ commit, state }, payload: BillingAccount): Promise<void> {
      if (state.current?.id === payload.id) {
        commit("current", payload);
      }
    },
    async create(
      { commit },
      payload: IBillingAccountCreateRequest,
    ): Promise<IBillingAccountCreateReply> {
      let obj = await BillingAccount.create(payload);
      commit("current", obj);
      return obj;
    },
  },
};
