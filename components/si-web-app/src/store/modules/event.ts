import Vue from "vue";
import { Module } from "vuex";
import _ from "lodash";

import { Event } from "@/api/sdf/model/event";
import { EventLog } from "@/api/sdf/model/eventLog";
import { OutputLine } from "@/api/sdf/model/outputLine";
import { RootStore } from "@/store";

export interface ActionSetContext {
  context: string[];
}

export interface ActionLoadLogs {
  eventId: string;
}

export interface ActionLoadOutputLines {
  eventLogId: string;
}

export interface EventStore {
  context: string[];
  list: Event[];
  logs: {
    [id: string]: EventLog[];
  };
  output: {
    [id: string]: OutputLine[];
  };
}

export const event: Module<EventStore, RootStore> = {
  namespaced: true,
  state: {
    context: [],
    list: [],
    logs: {},
    output: {},
  },
  mutations: {
    updateOutput(state, payload: OutputLine) {
      const currentList = state.output[payload.eventLogId] || [];
      const updateList = _.orderBy(
        _.unionBy([payload], currentList, "id"),
        ["unixTimestamp"],
        ["asc"],
      );
      Vue.set(state.output, payload.eventLogId, updateList);
    },
    updateLogs(state, payload: EventLog) {
      const currentList = state.logs[payload.eventId] || [];
      const updateList = _.orderBy(
        _.unionBy([payload], currentList, "id"),
        ["unixTimestamp"],
        ["asc"],
      );
      Vue.set(state.logs, payload.eventId, updateList);
    },
    updateList(state, payload: Event) {
      state.list = _.orderBy(
        _.unionBy([payload], state.list, "id"),
        ["startUnixTimestamp"],
        ["desc"],
      );
    },
    setOutputLines(
      state,
      payload: { eventLogId: string; outputLines: OutputLine[] },
    ) {
      Vue.set(
        state.output,
        payload.eventLogId,
        _.orderBy(payload.outputLines, ["unixTimestamp"], ["asc"]),
      );
    },
    setLogs(state, payload: { eventId: string; eventLogs: EventLog[] }) {
      Vue.set(
        state.logs,
        payload.eventId,
        _.orderBy(payload.eventLogs, ["unixTimestamp"], ["asc"]),
      );
    },
    setList(state, payload: Event[]) {
      state.list = _.orderBy(payload, ["startUnixTimestamp"], ["desc"]);
    },
    setContext(state, payload: string[]) {
      state.context = payload;
    },
    clear(state) {
      state.list = [];
      state.logs = {};
      state.output = {};
      state.context = [];
    },
  },
  actions: {
    async setContext({ commit }, payload: ActionSetContext) {
      commit("clear");
      commit("setContext", payload.context);
      const events = await Event.listForContext(payload.context);
      for (const event of events) {
        await event.loadOwner();
      }
      commit("setList", events);
    },
    async loadLogs({ commit }, payload: ActionLoadLogs) {
      const eventLogs = await EventLog.listForEvent(payload.eventId);
      commit("setLogs", { eventId: payload.eventId, eventLogs });
    },
    async loadOutputLines({ commit }, payload: ActionLoadOutputLines) {
      const outputLines = await OutputLine.listForEventLog(payload.eventLogId);
      commit("setOutputLines", { eventLogId: payload.eventLogId, outputLines });
    },
    async fromEvent({ state, commit }, payload: Event) {
      for (const c of state.context) {
        const inContext = _.indexOf(payload.context, c);
        if (inContext) {
          await payload.loadOwner();
          commit("updateList", payload);
          return;
        }
      }
    },
    async fromEventLog({ state, commit }, payload: EventLog) {
      if (_.find(state.list, ["id", payload.eventId])) {
        if (state.logs[payload.eventId]) {
          commit("updateLogs", payload);
        }
      }
    },
    async fromOutputLine({ state, commit }, payload: OutputLine) {
      if (_.find(state.list, ["id", payload.eventId])) {
        if (state.output[payload.eventLogId]) {
          commit("updateOutput", payload);
        }
      }
    },
  },
};
