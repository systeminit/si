import { Module } from "vuex";
import _ from "lodash";
import { Organization } from "@/api/sdf/model/organization";
import { Workspace } from "@/api/sdf/model/workspace";
import { Client, ClientObjectType, ClientKind } from "@/api/sdf/model/client";
import { RootStore } from "@/store";

export interface ClientStore {
  clients: Client[];
}

export interface ActionCreateClient {
  clientName: string;
  clientKind: ClientKind;
  message: Record<string, any>;
}

export const client: Module<ClientStore, RootStore> = {
  namespaced: true,
  state: {
    clients: [],
  },
  getters: {},
  mutations: {
    updateClients(state, payload: Client) {
      state.clients = _.unionBy([payload], state.clients, "id");
    },
  },
  actions: {
    async fromClient({ commit }, payload: Client) {
      commit("updateClients", payload);
    },
    async createClient(
      { commit, state, rootGetters },
      payload: ActionCreateClient,
    ) {

      const json = JSON.stringify(payload.message, null, 0);
      const message = new Uint8Array(json.length);
      for (let i = 0; i < json.length; i++) {
        message[i] = json.charCodeAt(i);
      }
      const organization: Organization = rootGetters["organization/current"];
      const workspace: Workspace = rootGetters["workspace/current"];

      const client = await Client.create({
        name: payload.clientName,
        objectType: ClientObjectType.Api,
        kind: payload.clientKind,
        message,
        organizationId: organization.id,
        workspaceId: workspace.id,
      });
      commit("updateClients", client);
    },
  },
};
