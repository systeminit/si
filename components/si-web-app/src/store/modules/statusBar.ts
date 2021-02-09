import Bottle from "bottlejs";
import _ from "lodash";
import { Module } from "vuex";
import { SiVuexStore } from "@/store";
import { PartyBus } from "@/api/partyBus";
import { CurrentChangeSetEvent } from "@/api/partyBus/currentChangeSetEvent";

export interface StatusBarStore {
  activatedBy: Set<string>;
  applicationName: string | null;
  systemName: string | null;
  nodeName: string | null;
  nodeType: string | null;
  changeSetName: string | null;
  editMode: "edit" | "view" | null;
}

export const statusBar: Module<StatusBarStore, any> = {
  namespaced: true,
  state(): StatusBarStore {
    return {
      activatedBy: new Set(),
      applicationName: null,
      systemName: null,
      nodeName: null,
      nodeType: null,
      changeSetName: null,
      editMode: null,
    };
  },
  mutations: {
    addToActivatedBy(state, payload: string) {
      state.activatedBy = state.activatedBy.add(payload);
    },
    removeFromActivatedBy(state, payload: string) {
      state.activatedBy.delete(payload);
    },
    clear(state) {
      state.applicationName = null;
      state.systemName = null;
      state.nodeName = null;
      state.nodeType = null;
      state.changeSetName = null;
      state.editMode = null;
    },
    setApplicationName(state, payload: StatusBarStore["applicationName"]) {
      state.applicationName = payload;
    },
    setSystemName(state, payload: StatusBarStore["systemName"]) {
      state.systemName = payload;
    },
    setNodeName(state, payload: StatusBarStore["nodeName"]) {
      state.nodeName = payload;
    },
    setNodeType(state, payload: StatusBarStore["nodeType"]) {
      state.nodeType = payload;
    },
    setChangeSetName(state, payload: StatusBarStore["changeSetName"]) {
      state.changeSetName = payload;
    },
    setEditMode(state, payload: StatusBarStore["editMode"]) {
      state.editMode = payload;
    },
  },
  actions: {
    activate({ commit }, payload: string) {
      commit("addToActivatedBy", payload);
    },
    deactivate({ commit, state }, payload: string) {
      commit("removeFromActivatedBy", payload);
      if (state.activatedBy.size == 0) {
        commit("clear");
      }
    },
    setApplicationName({ commit }, payload: StatusBarStore["applicationName"]) {
      commit("setApplicationName", payload);
    },
    setSystemName({ commit }, payload: StatusBarStore["systemName"]) {
      commit("setSystemName", payload);
    },
    setNodeName({ commit }, payload: StatusBarStore["nodeName"]) {
      commit("setNodeName", payload);
    },
    setNodeType({ commit }, payload: StatusBarStore["nodeType"]) {
      commit("setNodeType", payload);
    },
    setChangeSetName({ commit }, payload: StatusBarStore["changeSetName"]) {
      commit("setChangeSetName", payload);
    },
    setEditMode({ commit }, payload: boolean) {
      if (payload) {
        commit("setEditMode", "edit");
      } else {
        commit("setEditMode", "view");
      }
    },
    async onCurrentChangeSet({ dispatch }, event: CurrentChangeSetEvent) {
      await dispatch("setChangeSetName", event.changeSet?.name);
    },
  },
};

export function registerStatusBar(instanceId: string) {
  const bottle = Bottle.pop("default");
  const store: SiVuexStore = bottle.container.Store;
  if (!store.hasModule(["statusBar", instanceId])) {
    store.registerModule(["statusBar", instanceId], statusBar);
  }
  let partyBus: PartyBus = bottle.container.PartyBus;
  partyBus.subscribeToEvents("statusBar", instanceId, [CurrentChangeSetEvent]);
}

export function unregisterStatusBar(instanceId: string) {
  const bottle = Bottle.pop("default");
  const store: SiVuexStore = bottle.container.Store;
  if (store.hasModule(["statusBar", instanceId])) {
    store.unregisterModule(["statusBar", instanceId]);
  }
}
