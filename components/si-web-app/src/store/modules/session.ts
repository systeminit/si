import { Module } from "vuex";

import { User } from "@/api/sdf/model/user";
import { BillingAccount } from "@/api/sdf/model/billingAccount";
import { Organization } from "@/api/sdf/model/organization";
import { Workspace } from "@/api/sdf/model/workspace";
import { Entity } from "@/api/sdf/model/entity";

import {
  SessionDal,
  ISessionDalLoginRequest,
  ISessionDalLoginReply,
  IGetDefaultsReply,
} from "@/api/sdf/dal/sessionDal";
import { SDFError } from "@/api/sdf";
import { workspace$, system$ } from "@/observables";

export type ISetDefaultsReply = IGetDefaultsReply;

export enum ISessionContextKind {
  ApplicationSystem = "applicationSystem",
}

export interface ISessionContextApplicationSystem {
  kind: ISessionContextKind.ApplicationSystem;
  applicationId: string;
  systemId: string;
}

export type SessionContext = ISessionContextApplicationSystem;

export interface SessionStore {
  user: null | User;
  billingAccount: null | BillingAccount;
  currentWorkspace: null | Workspace;
  currentOrganization: null | Organization;
  currentSystem: null | Entity;
  sessionContext: null | SessionContext;
}

export const session: Module<SessionStore, any> = {
  namespaced: true,
  state: {
    user: null,
    billingAccount: null,
    currentWorkspace: null,
    currentOrganization: null,
    currentSystem: null,
    sessionContext: null,
  },
  mutations: {
    setUser(state, payload: SessionStore["user"]) {
      state.user = payload;
    },
    setBillingAccount(state, payload: SessionStore["billingAccount"]) {
      state.billingAccount = payload;
    },
    setCurrentOrganization(
      state,
      payload: SessionStore["currentOrganization"],
    ) {
      state.currentOrganization = payload;
    },
    setCurrentWorkspace(state, payload: SessionStore["currentWorkspace"]) {
      workspace$.next(payload);
      state.currentWorkspace = payload;
    },
    setCurrentSystem(state, payload: SessionStore["currentSystem"]) {
      system$.next(payload);
      state.currentSystem = payload;
    },
    setSessionContext(state, payload: SessionStore["sessionContext"]) {
      state.sessionContext = payload;
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
      commit("setCurrentWorkspace", null);
      commit("setCurrentOrganization", null);
      commit("setCurrentSystem", null);
    },
    async setDefaults({ commit }): Promise<ISetDefaultsReply> {
      const reply = await SessionDal.getDefaults();
      if (!reply.error) {
        commit("setCurrentOrganization", reply.organization);
        commit("setCurrentWorkspace", reply.workspace);
        commit("setCurrentSystem", reply.system);
      } else {
        console.log("error in set defaults", { reply });
      }
      return reply;
    },
    setSessionContext({ commit }, sessionContext: SessionContext | null) {
      commit("setSessionContext", sessionContext);
    },
  },
};
