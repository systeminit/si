import { Module } from "vuex";

import { ChangeSet } from "@/api/sdf/model/changeSet";
import { PartyBus } from "@/api/partyBus";
import { EditSession } from "@/api/sdf/model/editSession";
import Bottle from "bottlejs";
import { CurrentChangeSetEvent } from "@/api/partyBus/currentChangeSetEvent";
import { EditSessionCurrentSetEvent } from "@/api/partyBus/editSessionCurrentSetEvent";

import { changeSet$, editSession$, applicationId$ } from "@/observables";

export type IEditorContext = IEditorContextApplication;

export interface IEditorContextApplication {
  applicationId: string;
  contextType: "applicationSystem";
}

export interface EditorStore {
  version: number;
  context: IEditorContext | null;
  currentChangeSet: ChangeSet | null;
  currentEditSession: EditSession | null;
}

export function setupEditor() {
  const bottle = Bottle.pop("default");
  const partyBus: PartyBus = bottle.container.PartyBus;
  partyBus.subscribeToEvents("editor", undefined, [
    CurrentChangeSetEvent,
    EditSessionCurrentSetEvent,
  ]);
}

export const editor: Module<EditorStore, any> = {
  namespaced: true,
  state: {
    version: 1,
    currentChangeSet: null,
    currentEditSession: null,
    context: null,
  },
  getters: {
    inEditable(state): boolean {
      if (state.currentChangeSet && state.currentEditSession && state.context) {
        return true;
      } else {
        return false;
      }
    },
  },
  mutations: {
    setCurrentChangeSet(state, payload: EditorStore["currentChangeSet"]) {
      changeSet$.next(payload);
      state.currentChangeSet = payload;
    },
    setCurrentEditSession(state, payload: EditorStore["currentEditSession"]) {
      editSession$.next(payload);
      state.currentEditSession = payload;
    },
    setContext(state, payload: EditorStore["context"]) {
      applicationId$.next(payload?.applicationId);
      state.context = payload;
    },
  },
  actions: {
    async onCurrentChangeSet({ commit }, event: CurrentChangeSetEvent) {
      commit("setCurrentChangeSet", event.changeSet);
    },
    async onEditSessionCurrentSet(
      { commit },
      event: EditSessionCurrentSetEvent,
    ) {
      commit("setCurrentEditSession", event.editSession);
    },
    async setContext({ commit }, context: EditorStore["context"]) {
      commit("setContext", context);
    },
  },
};
