import Vue from "vue";
import { Module } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import { System } from "@/api/sdf/model/system";
import { RootStore } from "@/store";

export interface ActionSetCurrent {
  id: string;
}

export interface ActionSetSystem {
  id: string;
}

export interface ApplicationDetailStore {
  application: Entity | undefined;
  system: System | undefined;
  systems: System[];
}

export const applicationDetail: Module<ApplicationDetailStore, RootStore> = {
  namespaced: true,
  state: {
    application: undefined,
    system: undefined,
    systems: [],
  },
  getters: {},
  mutations: {
    application(state, payload: Entity | undefined) {
      state.application = payload;
    },
    system(state, payload: System | undefined) {
      state.system = payload;
    },
    setSystems(state, payload: System[]) {
      state.systems = payload;
    },
    clear(state) {
      state.application = undefined;
    },
  },
  actions: {
    async clear({ commit }) {
      commit("clear");
    },
    async setApplication({ commit }, payload: ActionSetCurrent) {
      let application = await Entity.get_head(payload);
      let systems = await application.systems();
      commit("application", application);
      commit("setSystems", systems);
      commit("system", systems[0]);
    },
    async setSystem({ commit }, payload: ActionSetSystem) {
      let system = await System.get(payload);
      commit("system", system);
    },
  },
};
