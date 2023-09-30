import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";

import { useAuthStore } from "@/store/auth.store";
import { useChangeSetsStore, ChangeSetId } from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { LabelList } from "@/api/sdf/dal/label_list";
import { ActorAndTimestamp } from "./components.store";

export type SecretId = string;
export type SecretDefinitionId = string;

export type Secret = {
  id: SecretId;
  definition: SecretDefinitionId;
  name: string;
  description?: string;
  createdInfo: ActorAndTimestamp;
  updatedInfo?: ActorAndTimestamp;
  expiration?: string;
};

export interface SecretFormSchema {
  name: string;
  kind: string;
  widgetKind: string;
  widgetOptions: null | LabelList<string | number>;
}

export interface SecretDefinitionView {
  secretDefinition: SecretDefinitionId;
  formData: SecretFormSchema[];
}

export interface SecretDefinitionValue {
  id: string;
  displayName: string;
  value: unknown;
}

export interface SecretDefinition {
  fields: { [id: string]: SecretDefinitionValue };
}

export type SecretsHashMap = Record<SecretDefinitionId, Secret[]>;
export type SecretsDefinitionHashMap = Record<
  SecretDefinitionId,
  {
    definition: SecretDefinitionView;
    secrets: Secret[];
  }
>;

export function useSecretsStore() {
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();

  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  const changeSetId: ChangeSetId | null = changeSetsStore.selectedChangeSetId;

  // TODO: probably these should be passed in automatically
  // and need to make sure it's done consistently (right now some endpoints vary slightly)
  const visibilityParams = {
    visibility_change_set_pk: changeSetId,
    workspaceId,
  };

  return addStoreHooks(
    defineStore("secrets", {
      state: () => ({
        secretDefinitionByDefinitionId: {} as SecretsDefinitionHashMap,
        secretIsTransitioning: {} as Record<SecretId, boolean>,
      }),
      getters: {
        secretsByDefinitionId(state): SecretsHashMap {
          return _.transform(
            state.secretDefinitionByDefinitionId,
            (acc, value, key) => {
              acc[key] = value.secrets;
            },
          );
        },
        secrets(): Secret[] {
          return _.flatMap(this.secretsByDefinitionId);
        },
        secretsById(): Record<SecretId, Secret> {
          return _.keyBy(this.secrets, (s) => s.id);
        },
        definitions: (state) => _.keys(state.secretDefinitionByDefinitionId),
        secretFormSchemaByDefinitionId(
          state,
        ): Record<SecretDefinitionId, SecretFormSchema[]> {
          return _.transform(
            state.secretDefinitionByDefinitionId,
            (acc, value, key) => {
              acc[key] = value.definition.formData;
            },
          );
        },
      },
      actions: {
        async LOAD_SECRETS() {
          return new ApiRequest<SecretsDefinitionHashMap>({
            url: "secret",
            params: {
              ...visibilityParams,
            },
            onSuccess: (response) => {
              this.secretDefinitionByDefinitionId = response;
            },
          });
        },
        async SAVE_SECRET(
          definition: SecretDefinitionId,
          name: string,
          value: Record<string, string>,
          description?: string,
          expiration?: string,
        ) {
          if (_.isEmpty(name)) {
            throw new Error("All secrets must have a name.");
          }

          if (this.secretsByDefinitionId[definition] === undefined) {
            throw new Error(
              "All secrets must be created based on a definition.",
            );
          }

          const user = useAuthStore().user;

          if (_.isNil(user)) {
            throw new Error("All secrets must be created by a specific user.");
          }

          const tempId = `-${Math.floor(
            Math.random() * 899999 + 100000,
          ).toString()}`;

          return new ApiRequest<Secret>({
            method: "post",
            url: "secret",
            params: {
              ...visibilityParams,
              name,
              description,
              definition,
              expiration,
            },
            optimistic: () => {
              const { pk: userId, name: userName } = user;

              this.secretsByDefinitionId[definition]?.push({
                id: tempId,
                definition,
                name,
                description,
                createdInfo: {
                  actor: { kind: "user", label: userName, id: userId },
                  timestamp: Date(),
                },
                expiration,
              });
              this.secretIsTransitioning[tempId] = true;

              return () => {
                const secretsOnDef = this.secretsByDefinitionId[definition];
                if (secretsOnDef === undefined) return;

                this.secretsByDefinitionId[definition] = secretsOnDef.filter(
                  (s) => s.id !== tempId,
                );
                this.secretIsTransitioning[tempId] = false;
              };
            },
            onSuccess: (response) => {
              const secretsOnDef = this.secretsByDefinitionId[definition];
              if (secretsOnDef === undefined) return;

              this.secretsByDefinitionId[definition] = secretsOnDef.map((s) =>
                s.id === tempId ? response : s,
              );
              this.secretIsTransitioning[tempId] = false;
            },
          });
        },
        async DELETE_SECRET(id: SecretId) {
          const secret = this.secretsById[id];

          if (_.isNil(secret)) return;

          return new ApiRequest({
            method: "delete",
            url: "secret",
            params: {
              ...visibilityParams,
              id,
            },
            optimistic: () => {
              this.secretIsTransitioning[secret.id] = true;

              return () => {
                this.secretIsTransitioning[secret.id] = false;
              };
            },
            onSuccess: () => {
              const secretsOnDef =
                this.secretsByDefinitionId[secret.definition];
              if (secretsOnDef === undefined) return;

              this.secretsByDefinitionId[secret.definition] =
                secretsOnDef.filter((s) => s.id !== id);

              this.secretIsTransitioning[secret.id] = false;
            },
          });
        },
      },
      onActivated() {
        // TODO Run load secrets on websocket message too
        this.LOAD_SECRETS();
      },
    }),
  )();
}
