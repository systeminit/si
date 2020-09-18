import { Module } from "vuex";
import _ from "lodash";

//import { BillingAccount, BillingAccountGetReply } from "@/graphql-types";
import { RootStore } from "@/store";
//import { graphqlQueryListAll, graphqlQuery } from "@/api/apollo";

import {
  BillingAccount,
  IBillingAccountCreateRequest,
  IBillingAccountCreateReply,
} from "@/api/sdf/model/billingAccount";

export interface BillingAccountStore {
  current: null | BillingAccount;
}

interface AddMutation {
  billingAccounts: BillingAccount[];
}

interface GetAction {
  billingAccountId: string;
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
    //async get({ commit }, payload: GetAction): Promise<void> {
    //  if (!payload.billingAccountId) {
    //    throw new Error(
    //      `cannot get a billing account without a valid id: ${payload.billingAccountId}`,
    //    );
    //  }
    //  const billingAccount: IBillingAccountGetReply = await graphqlQuery({
    //    typeName: "billingAccount",
    //    methodName: "get",
    //    variables: {
    //      id: payload.billingAccountId,
    //    },
    //  });
    //  commit("add", { billingAccounts: [billingAccount.item] });
    //  commit("current", billingAccount.item);
    //},
  },
};
