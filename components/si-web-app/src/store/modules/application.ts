import { Module } from "vuex";
import _ from "lodash";

import { ApplicationEntity } from "@/graphql-types";
import { RootStore } from "@/store";

export interface ApplicationStore {
  applications: ApplicationEntity[];
  current: null | ApplicationEntity;
}

interface AddMutation {
  applications: ApplicationEntity[];
}

interface CreateMutation {
  name: string;
}

interface GetGetter {
  id: string;
}

export const application: Module<ApplicationStore, RootStore> = {
  namespaced: true,
  state: {
    applications: [],
    current: null,
  },
  getters: {
    current(state): ApplicationEntity {
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current application; it is not set!");
      }
    },
    saved(state): ApplicationEntity[] {
      return _.filter(state.applications, entity => {
        if (!entity.siStorable?.changeSetId) {
          return true;
        } else {
          return false;
        }
      });
    },
    // prettier-ignore
    get: (state) => (filter: GetGetter): ApplicationEntity => {
      const app = _.find(state.applications, ["id", filter.id]);
      if (app) {
        return app;
      } else {
        throw new Error(`cannot find application id ${filter.id}`);
      }
    }
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.applications = _.unionBy(
        payload.applications,
        state.applications,
        "id",
      );
    },
    current(state, payload: ApplicationEntity) {
      state.current = payload;
    },
  },
  actions: {
    setCurrentById({ commit, state }, applicationId: string) {
      let app = _.find(state.applications, ["id", applicationId]);
      if (app) {
        commit("current", app);
      } else {
        console.log("cannot find application", {
          applications: state.applications,
        });
        throw new Error(`cannot find application for ${applicationId}`);
      }
    },
    add({ commit }, payload: AddMutation) {
      commit("add", payload);
    },
    async create(
      { dispatch, rootGetters, commit },
      payload: CreateMutation,
    ): Promise<ApplicationEntity> {
      await dispatch("changeSet/createDefault", {}, { root: true });
      let currentSystem = rootGetters["system/current"];
      let newApp = await dispatch(
        "entity/create",
        {
          typeName: "application_entity",
          data: {
            name: payload.name,
            properties: { inSystems: [currentSystem.id] },
          },
        },
        { root: true },
      );
      await dispatch("changeSet/execute", { wait: true }, { root: true });
      const toCommit = _.cloneDeep(newApp);
      toCommit.id = toCommit.siStorable.itemId;
      commit("add", { applications: [toCommit] });
      return newApp;
    },
  },
};
