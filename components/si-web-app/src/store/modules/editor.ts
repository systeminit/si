import { Module } from "vuex";
import _ from "lodash";

import { RootStore } from "@/store";

export interface EditorStore {
  mode: "view" | "edit";
  isSaving: boolean;
  editSaveError: undefined | Error;
}

export const editor: Module<EditorStore, RootStore> = {
  namespaced: true,
  state: {
    mode: "view",
    isSaving: false,
    editSaveError: undefined,
  },
  actions: {
    modeSwitch({ commit, state, rootState, dispatch }) {
      if (state.mode == "view") {
        if (rootState.changeSet.current) {
          commit("setMode", "edit");
        } else {
          dispatch("changeSet/createDefault", {}, { root: true }).then(() => {
            commit("setMode", "edit");
          });
        }
      } else {
        commit("setMode", "view");
      }
    },
  },
  mutations: {
    setIsSaving(state, saving: boolean) {
      state.isSaving = saving;
    },
    setEditSaveError(state, error: Error) {
      state.editSaveError = error;
    },
    setMode(state, mode: EditorStore["mode"]) {
      state.mode = mode;
    },
  },
};
