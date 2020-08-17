import { Module } from "vuex";
import _ from "lodash";

import { System, ApplicationEntity } from "@/graphql-types";
import { RootStore } from "@/store";
import { graphqlMutation, graphqlQueryListAll } from "@/api/apollo";

export interface SystemStore {
  systems: System[];
  current: null | System;
}

interface AddMutation {
  systems: System[];
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
    current(state): System {
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current system; it is not set!");
      }
    },
    saved(state): System[] {
      return _.filter(state.systems, entity => {
        if (!entity.siStorable?.changeSetId) {
          return true;
        } else {
          return false;
        }
      });
    },
    // prettier-ignore
    byId: (state: SystemStore) => (systemId: string): System | null => {
      let system = _.find(state.systems, ["id", systemId]);
      if (system) {
        return system;
      } else {
        return null;
      }
    },
    // prettier-ignore
    forApplicationId: (state, _getters, _rootState, rootGetters) => (applicationId: string): SystemStore["systems"] => {
      const application: ApplicationEntity = rootGetters["application/get"]({ "id":  applicationId });
      if (application) {
        const results: System[] = _.filter(state.systems, (system: System) => {
          if (_.find(application.properties?.inSystems, (f) => f == system.id)) {
            return true;
          } else {
            return false;
          }
        });
        return results;
      } else {
        return state.systems;
      }
    }
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.systems = _.unionBy(payload.systems, state.systems, "id");
    },
    current(state, payload: System) {
      state.current = payload;
    },
  },
  actions: {
    add({ commit }, payload: AddMutation) {
      commit("add", payload);
    },
    async createDefault({ state, commit, rootGetters }) {
      if (!_.find(state.systems, ["name", "default"])) {
        const workspace = rootGetters["workspace/current"];
        const profile = rootGetters["user/profile"];
        let system = await graphqlMutation({
          typeName: "system",
          methodName: "create",
          variables: {
            name: "default",
            displayName: "default",
            siProperties: {
              workspaceId: workspace.id,
              billingAccountId: profile.billingAccount?.id,
              organizationId: profile.organization?.id,
            },
          },
        });
        commit("add", { systems: [system.item] });
      }
    },
    async setCurrentToDefault({ state, commit }) {
      const defaultSystem = _.find(state.systems, system => {
        if (system.name == "default") {
          return true;
        } else {
          return false;
        }
      });
      if (defaultSystem) {
        commit("current", defaultSystem);
      }
    },
    setCurrentById({ commit, getters }, systemId: string | null) {
      if (systemId) {
        const system = getters["byId"](systemId);
        commit("current", system);
      } else {
        commit("current", null);
      }
    },
    async load({ commit, dispatch }): Promise<void> {
      const systems: System[] = await graphqlQueryListAll({
        typeName: "system",
      });
      if (systems.length > 0) {
        commit("add", { systems });
      } else {
        // NOTE: this should be pushed into billing account creation!
        await dispatch("createDefault");
      }
      await dispatch("setCurrentToDefault");
    },
  },
};
