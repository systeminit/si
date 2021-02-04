import { Module } from "vuex";

import { GetCurrentError } from "@/store";
import { Organization } from "@/api/sdf/model/organization";

export interface OrganizationStore {
  current: null | Organization;
}

export const organization: Module<OrganizationStore, any> = {
  namespaced: true,
  state: {
    current: null,
  },
  getters: {
    current(state): Organization {
      if (state.current) {
        return state.current;
      } else {
        throw new GetCurrentError("organization");
      }
    },
  },
  mutations: {
    current(state, payload: Organization | null) {
      state.current = payload;
    },
  },
  actions: {
    async default({ commit, state }): Promise<Organization> {
      const orgs = await Organization.find("name", "default");
      if (orgs.length) {
        commit("current", orgs[0]);
      } else {
        throw new Error("cannot find default organization");
      }
      // @ts-ignore - we know you think it could be null, but it can't!
      return state.current;
    },
    async fromDb({ commit, state }, payload: Organization): Promise<void> {
      if (state.current?.id === payload.id) {
        commit("current", payload);
      }
    },
  },
};
