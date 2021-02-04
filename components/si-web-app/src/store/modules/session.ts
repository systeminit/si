import { Module } from "vuex";

import { User } from "@/api/sdf/model/user";
import { BillingAccount } from "@/api/sdf/model/billingAccount";
import {
  SessionDal,
  ISessionDalLoginRequest,
  ISessionDalLoginReply,
} from "@/api/sdf/dal/sessionDal";
import { SDFError } from "@/api/sdf";

export interface SessionStore {
  user: null | User;
  billingAccount: null | BillingAccount;
}

export const session: Module<SessionStore, any> = {
  namespaced: true,
  state: {
    user: null,
    billingAccount: null,
  },
  mutations: {
    setUser(state, payload: SessionStore["user"]) {
      state.user = payload;
    },
    setBillingAccount(state, payload: SessionStore["billingAccount"]) {
      state.billingAccount = payload;
    },
  },
  actions: {
    async isAuthenticated({
      dispatch,
      state,
      commit,
    }): Promise<SDFError | boolean> {
      let reply = await SessionDal.isAuthenticated({
        user: state.user,
        billingAccount: state.billingAccount,
      });
      if (reply.error) {
        return reply.error;
      } else if (reply.logout) {
        await dispatch("logout");
        return false;
      } else if (reply.login) {
        return false;
      } else {
        commit("setUser", reply.user);
        commit("setBillingAccount", reply.billingAccount);
        return true;
      }
    },
    async login(
      { commit },
      request: ISessionDalLoginRequest,
    ): Promise<ISessionDalLoginReply> {
      const reply = await SessionDal.login(request);
      if (!reply.error) {
        commit("setUser", reply.user);
        commit("setBillingAccount", reply.billingAccount);
      }
      return reply;
    },
    async logout({ dispatch }): Promise<void> {
      await SessionDal.logout();
      await dispatch("clear");
    },
    async clear({ commit }) {
      commit("setUser", null);
      commit("setBillingAccount", null);
    },
  },
};
