import { Module } from "vuex";
import _ from "lodash";
import { Organization } from "@/api/sdf/model/organization";
import { Workspace } from "@/api/sdf/model/workspace";
import { Secret, SecretObjectType, SecretKind } from "@/api/sdf/model/secret";
import { PublicKey } from "@/api/sdf/model/keyPair";
import { RootStore } from "@/store";

export interface SecretStore {
  secrets: Secret[];
  publicKey: PublicKey | undefined;
}

export interface ActionCreateCredential {
  secretName: string;
  secretKind: SecretKind;
  message: Record<string, any>;
}

export const secret: Module<SecretStore, RootStore> = {
  namespaced: true,
  state: {
    secrets: [],
    publicKey: undefined,
  },
  getters: {},
  mutations: {
    publicKey(state, payload: PublicKey | undefined) {
      state.publicKey = payload;
    },
    updateSecrets(state, payload: Secret) {
      state.secrets = _.unionBy([payload], state.secrets, "id");
    },
  },
  actions: {
    async fromPublicKey({ commit }, payload: PublicKey) {
      commit("publicKey", payload);
    },
    async fromSecret({ commit }, payload: Secret) {
      commit("updateSecrets", payload);
    },
    async createCredential(
      { commit, state, rootGetters },
      payload: ActionCreateCredential,
    ) {
      if (!state.publicKey) {
        throw new Error("publicKey not set");
      }

      const json = JSON.stringify(payload.message, null, 0);
      const message = new Uint8Array(json.length);
      for (let i = 0; i < json.length; i++) {
        message[i] = json.charCodeAt(i);
      }
      const organization: Organization = rootGetters["organization/current"];
      const workspace: Workspace = rootGetters["workspace/current"];

      const secret = await Secret.create({
        name: payload.secretName,
        objectType: SecretObjectType.Credential,
        kind: payload.secretKind,
        message,
        publicKey: state.publicKey,
        organizationId: organization.id,
        workspaceId: workspace.id,
      });
      commit("updateSecrets", secret);
    },
  },
};
