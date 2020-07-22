import { Module } from "vuex";
import _ from "lodash";

import { SystemEntity, EdgeEntity } from "@/graphql-types";
import { RootStore } from "@/store";

export interface SystemStore {
  systems: SystemEntity[];
  current: null | SystemEntity;
}

interface AddMutation {
  systems: SystemEntity[];
}

interface CreateMutation {
  name: string;
}

export const system: Module<SystemStore, RootStore> = {
  namespaced: true,
  state: {
    systems: [],
    current: null,
  },
  getters: {
    current(state): SystemEntity {
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current system; it is not set!");
      }
    },
    saved(state): SystemEntity[] {
      return _.filter(state.systems, entity => {
        if (!entity.siStorable?.changeSetId) {
          return true;
        } else {
          return false;
        }
      });
    },
    // prettier-ignore
    forApplicationId: (state, _getters, _rootState, rootGetters) => (applicationId: string): SystemStore["systems"] => {
      let edges = rootGetters["edge/fromIdForType"]({id: applicationId, typeName: "system_entity"});
      // @ts-ignore
      const results: SystemEntity[] = _.filter(state.systems, (system: SystemEntity) => {
        for (const edge of edges) {
          if (edge.properties.headVertex.typeName == "system_entity") {
            return system.id == edge.properties.headVertex.id;
          } else if (edge.properties.tailVertex.typeName == "system_entity") {
            return system.id == edge.properties.tailVertex.id;
          } else {
            return false;
          }
        }
      });
      return results;
    }
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.systems = _.unionBy(payload.systems, state.systems, "id");
    },
    current(state, payload: SystemEntity) {
      state.current = payload;
    },
  },
  actions: {
    add({ commit }, payload: AddMutation) {
      commit("add", payload);
    },
    async createDefault({ state, dispatch }) {
      if (!_.find(state.systems, ["name", "default"])) {
        await dispatch("changeSet/createDefault", {}, { root: true });
        await dispatch(
          "entity/create",
          {
            typeName: "system_entity",
            data: {
              name: "default",
            },
          },
          { root: true },
        );
        await dispatch("changeSet/execute", { wait: true }, { root: true });
      }
    },
    async setCurrentToDefault({ state, commit }) {
      const defaultSystem = _.find(state.systems, system => {
        if (system.name == "default" && !system.siStorable?.changeSetId) {
          return true;
        } else {
          return false;
        }
      });
      if (defaultSystem) {
        commit("current", defaultSystem);
      }
    },
  },
};
