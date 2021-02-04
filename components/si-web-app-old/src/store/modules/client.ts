import { Module } from "vuex";
import _ from "lodash";
import { ApiClient, ApiClientKind } from "@/api/sdf/model/apiClient";
import { RootStore } from "@/store";

export interface ClientStore {
  clients: ApiClient[];
}

export interface ActionCreateClient {
  name: string;
  kind: ApiClientKind;
}

export const client: Module<ClientStore, RootStore> = {
  namespaced: true,
  state: {
    clients: [],
  },
  getters: {},
  mutations: {
    updateClients(state, payload: ApiClient) {
      state.clients = _.orderBy(
        _.unionBy([payload], state.clients, "id"),
        ["kind", "name"],
        ["asc", "asc"],
      );
    },
    setClients(state, payload: ApiClient[]) {
      state.clients = _.orderBy(payload, ["kind", "name"], ["asc", "asc"]);
    },
  },
  actions: {
    async loadClients({ commit }) {
      let clients = await ApiClient.list({ pageSize: 500 });
      commit("setClients", clients.items);
    },
    async fromApiClient({ commit }, payload: ApiClient) {
      commit("updateClients", payload);
    },
    async createClient(
      { commit },
      payload: ActionCreateClient,
    ): Promise<string> {
      const apiClientResponse = await ApiClient.create(payload);
      commit("updateClients", apiClientResponse.apiClient);
      return apiClientResponse.token;
    },
  },
};
