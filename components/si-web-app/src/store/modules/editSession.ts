import { Module } from "vuex";
import _ from "lodash";

import { EditSession } from "@/graphql-types";
import { RootStore } from "@/store";
import { graphqlMutation, graphqlQueryListAll } from "@/api/apollo";

export interface EditSessionStore {
  editSessions: EditSession[];
  current: null | EditSession;
}

interface AddMutation {
  editSessions: EditSessionStore["editSessions"];
}

export const editSession: Module<EditSessionStore, RootStore> = {
  namespaced: true,
  state: {
    editSessions: [],
    current: null,
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.editSessions = _.unionBy(
        payload.editSessions,
        state.editSessions,
        "id",
      );
    },
    current(state, payload: EditSessionStore["current"]) {
      state.current = payload;
    },
  },
  getters: {
    current(state): EditSession {
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current editSession; it is not set!");
      }
    },
  },
  actions: {
    add({ commit }, payload: AddMutation) {
      commit("add", payload);
    },
    async revert({ commit, getters }): Promise<EditSession> {
      const editSession = getters["current"];
      const resultEditSession = await graphqlMutation({
        typeName: "editSession",
        methodName: "revert",
        variables: {
          id: editSession.id,
        },
      });
      commit("add", { editSessions: [resultEditSession.item] });
      commit("current", resultEditSession.item);
      return resultEditSession.item;
    },
    async create({ commit, rootGetters }): Promise<EditSession> {
      const workspace = rootGetters["workspace/current"];
      const profile = rootGetters["user/profile"];
      const time = new Date().toUTCString();
      const changeSet = rootGetters["changeSet/current"];
      const editSession = await graphqlMutation({
        typeName: "editSession",
        methodName: "create",
        variables: {
          name: `${profile.user?.name} ${time}`,
          displayName: `${profile.user?.name} ${time}`,
          siProperties: {
            workspaceId: workspace.id,
            billingAccountId: profile.billingAccount?.id,
            organizationId: profile.organization?.id,
            changeSetId: changeSet.id,
            userId: profile.user?.id,
          },
          reverted: false,
        },
      });
      commit("add", { editSessions: [editSession.item] });
      commit("current", editSession.item);
      return editSession.item;
    },
    async load({ commit }): Promise<void> {
      const editSessions: EditSession[] = await graphqlQueryListAll({
        typeName: "editSession",
      });
      if (editSessions.length > 0) {
        commit("add", { editSession });
      }
    },
  },
};
