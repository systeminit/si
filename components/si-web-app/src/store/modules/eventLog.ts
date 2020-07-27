import { Module } from "vuex";
import _ from "lodash";

import { EventLog } from "@/graphql-types";
import { RootStore } from "@/store";
import { graphqlQueryListAll } from "@/api/apollo";

export interface EventLogStore {
  eventLogs: EventLog[];
}

interface AddMutation {
  eventLogs: EventLog[];
}

export const eventLog: Module<EventLogStore, RootStore> = {
  namespaced: true,
  state: {
    eventLogs: [],
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.eventLogs = _.orderBy(
        _.unionBy(payload.eventLogs, state.eventLogs, "id"),
        ["timestamp"],
        ["desc"],
      );
    },
  },
  getters: {
    latest(state): EventLog | null {
      if (state.eventLogs[0]) {
        return state.eventLogs[0];
      } else {
        return null;
      }
    },
  },
  actions: {
    add({ commit }, payload: AddMutation) {
      commit("add", payload);
    },
    async load({ commit }): Promise<void> {
      const eventLogs: EventLog[] = await graphqlQueryListAll({
        typeName: "eventLog",
      });
      if (eventLogs.length > 0) {
        commit("add", { eventLogs });
      }
    },
  },
};
