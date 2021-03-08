import { Module } from "vuex";
import _ from "lodash";
import sealedBox from "tweetnacl-sealedbox-js";
import {
  ICreateSecretReply,
  ICreateSecretRequest as DalISecretCreateRequest,
  SecretDal,
  IListSecretsRequest,
  IListSecretsReply,
} from "@/api/sdf/dal/secretDal";
import {
  SecretKind,
  SecretVersion,
  SecretAlgorithm,
  ISecret,
} from "@/api/sdf/model/secret";

export type ISetSecretListRequest = IListSecretsRequest;
export type ISetSecretListReply = IListSecretsReply;

export interface ISecretCreateRequest {
  name: string;
  kind: SecretKind;
  message: Record<string, string>;
}

export interface SecretStore {
  secretList: ISecret[];
}

export const secret: Module<SecretStore, any> = {
  namespaced: true,
  state: {
    secretList: [],
  },
  mutations: {
    setSecretList(state, payload: ISecret[]) {
      state.secretList = payload;
    },
    updateSecretList(state, payload: ISecret) {
      state.secretList = _.unionBy([payload], state.secretList, "id");
    },
  },
  actions: {
    async createSecret(
      { commit, rootState },
      request: ISecretCreateRequest,
    ): Promise<ICreateSecretReply> {
      const pkReply = await SecretDal.getPublicKey();
      if (pkReply.error) {
        return pkReply;
      }
      const publicKey = pkReply.publicKey;

      const crypted = encryptMessage(request.message, publicKey.publicKey);

      const dalRequest: DalISecretCreateRequest = {
        name: request.name,
        objectType: SecretKind.objectTypeFor(request.kind),
        kind: request.kind,
        crypted,
        keyPairId: publicKey.id,
        version: SecretVersion.defaultValue(),
        algorithm: SecretAlgorithm.defaultValue(),
        workspaceId: rootState.session.currentWorkspace.id,
      };

      const reply = await SecretDal.createSecret(dalRequest);
      if (!reply.error) {
        commit("updateSecretList", reply.secret);
      }
      return reply;
    },
    async setSecretList(
      { commit },
      request: ISetSecretListRequest,
    ): Promise<ISetSecretListReply> {
      const reply = await SecretDal.listSecrets(request);
      if (!reply.error) {
        commit("setSecretList", reply.list);
      }
      return reply;
    },
  },
};

function encryptMessage(
  message: Record<string, string>,
  publicKey: Uint8Array,
): number[] {
  return Array.from(sealedBox.seal(serializeMessage(message), publicKey));
}

function serializeMessage(message: Record<string, string>): Uint8Array {
  const json = JSON.stringify(message, null, 0);

  const result = new Uint8Array(json.length);
  for (let i = 0; i < json.length; i++) {
    result[i] = json.charCodeAt(i);
  }

  return result;
}
