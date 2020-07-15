import { Module } from "vuex";
import _ from "lodash";

import { BillingAccount, BillingAccountGetReply } from "@/graphql-types";
import { RootStore } from "@/store";
import { graphqlQueryListAll, graphqlQuery } from "@/api/apollo";

export interface BillingAccountStore {
  billingAccounts: BillingAccount[];
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
    billingAccounts: [],
    current: null,
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.billingAccounts = _.unionBy(
        payload.billingAccounts,
        state.billingAccounts,
        "id",
      );
    },
    current(state, payload: BillingAccount) {
      state.current = payload;
    },
  },
  actions: {
    async get({ commit }, payload: GetAction): Promise<void> {
      if (!payload.billingAccountId) {
        throw new Error(
          `cannot get a billing account without a valid id: ${payload.billingAccountId}`,
        );
      }
      const billingAccount: BillingAccountGetReply = await graphqlQuery({
        typeName: "billingAccount",
        methodName: "get",
        variables: {
          id: payload.billingAccountId,
        },
      });
      commit("add", { billingAccounts: [billingAccount.item] });
      commit("current", billingAccount.item);
    },
  },
};
