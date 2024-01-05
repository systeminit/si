import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";

import { useAuthStore } from "@/store/auth.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { encryptMessage } from "@/utils/messageEncryption";
import { PropertyEditorPropWidgetKind } from "@/api/sdf/dal/property_editor";
import { ActorAndTimestamp } from "./components.store";
import { useRealtimeStore } from "./realtime/realtime.store";

/**
 * A public key with metadata, used to encrypt secrets
 */
export interface PublicKey {
  /**
   * The PK of the public key
   */
  pk: string;
  /**
   * The name of the public key
   */
  name: string;
  /**
   * The public key contents, encoded as a Base64 string
   *
   * # Examples
   *
   * Decoding a public key into a `Uint8Array` type:
   *
   * ```ts
   * Base64.toUint8Array(key.public_key);
   * ```
   */
  public_key: string;
  /**
   * A created lamport clock, used to sort multiple generations of key pairs
   */
  created_lamport_clock: string;

  created_at: string;
  updated_at: string;
}

export enum SecretVersion {
  V1 = "v1",
}

export enum SecretAlgorithm {
  Sealedbox = "sealedbox",
}

export type SecretId = string;
export type SecretDefinitionId = string;

// TODO: Store description on the secrets table
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
  widgetKind: PropertyEditorPropWidgetKind;
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
export type SecretsOrderedArray = {
  id: SecretDefinitionId;
  latestChange: string | undefined;
  secrets: Secret[];
}[];
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
  const changeSetId = changeSetsStore.selectedChangeSetId;

  // TODO: probably these should be passed in automatically
  // and need to make sure it's done consistently (right now some endpoints vary slightly)
  const visibilityParams = {
    visibility_change_set_pk: changeSetId,
    workspaceId,
  };

  return addStoreHooks(
    defineStore(
      `ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/secrets`,
      {
        state: () => ({
          secretDefinitionByDefinitionId: {} as SecretsDefinitionHashMap,
          // Stores whether a request is running on a secret. Previously used to disable the delete button temporarily
          secretIsTransitioning: {} as Record<SecretId, boolean>,
          publicKey: null as PublicKey | null,
        }),
        getters: {
          secretsByDefinitionId(state): SecretsHashMap {
            return _.mapValues(
              state.secretDefinitionByDefinitionId,
              (s) => s.secrets,
            );
          },
          secretsByLastCreated(state): SecretsOrderedArray {
            const out = [] as SecretsOrderedArray;

            for (const [key, value] of Object.entries(
              state.secretDefinitionByDefinitionId,
            )) {
              let change: string | undefined;
              value.secrets.forEach((secret) => {
                if (!change) change = secret.createdInfo.timestamp;
                if (new Date(secret.createdInfo.timestamp) > new Date(change)) {
                  change = secret.createdInfo.timestamp;
                }
                if (
                  secret.updatedInfo &&
                  new Date(secret.updatedInfo.timestamp) > new Date(change)
                ) {
                  change = secret.updatedInfo.timestamp;
                }
              });

              out.push({
                id: key,
                latestChange: change,
                secrets: value.secrets,
              });
            }

            out.sort((a, b) => {
              if (!a.latestChange && !b.latestChange) return 0;
              else if (!a.latestChange && b.latestChange) return 1;
              else if (a.latestChange && !b.latestChange) return -1;
              else if (a.latestChange && b.latestChange) {
                if (new Date(a.latestChange) > new Date(b.latestChange)) {
                  return -1;
                } else if (
                  new Date(a.latestChange) < new Date(b.latestChange)
                ) {
                  return 1;
                }
              }
              return 0;
            });

            return out;
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
          async UPDATE_SECRET(secret: Secret, value?: Record<string, string>) {
            if (_.isEmpty(secret.name)) {
              throw new Error("All secrets must have a name.");
            }

            if (this.secretsByDefinitionId[secret.definition] === undefined) {
              throw new Error(
                "All secrets must be created based on a definition.",
              );
            }

            const user = useAuthStore().user;

            if (_.isNil(user)) {
              throw new Error(
                "All secrets must be created by a specific user.",
              );
            }

            const { id, name, definition, description, createdInfo } = secret;
            const params = {
              ...visibilityParams,
              id,
              name,
              description,
              newSecretData: null,
            } as {
              visibility_change_set_pk: string | null;
              workspaceId: string | null;
              name: string;
              description: string;
              newSecretData: {
                crypted: number[];
                keyPairPk: string;
                version: SecretVersion;
                algorithm: SecretAlgorithm;
              } | null;
            };

            if (value) {
              // Encrypt Value
              if (_.isNil(this.publicKey)) {
                throw new Error("Couldn't fetch publicKey.");
              }

              const algorithm = SecretAlgorithm.Sealedbox;
              const version = SecretVersion.V1;

              const keyPairPk = this.publicKey.pk;

              const crypted = await encryptMessage(value, this.publicKey);

              params.newSecretData = {
                algorithm,
                version,
                keyPairPk,
                crypted,
              };
            }

            return new ApiRequest<Secret>({
              method: "patch",
              url: "secret",
              params,
              optimistic: () => {
                const { pk: userId, name: userName } = user;

                this.secretDefinitionByDefinitionId[
                  secret.definition
                ]?.secrets?.filter((s) => {
                  return s.id !== secret.id;
                });
                this.secretDefinitionByDefinitionId[
                  secret.definition
                ]?.secrets?.push({
                  id,
                  name,
                  definition,
                  description,
                  createdInfo,
                  updatedInfo: {
                    actor: { kind: "user", label: userName, id: userId },
                    timestamp: Date(),
                  },
                });
                this.secretIsTransitioning[id] = true;

                return () => {
                  const definition = this.secretDefinitionByDefinitionId[id];

                  if (definition === undefined) return;

                  definition.secrets = definition.secrets.filter(
                    (s) => s.id !== id,
                  );
                  this.secretIsTransitioning[id] = false;
                };
              },
              onSuccess: (response) => {
                const definition = this.secretDefinitionByDefinitionId[id];

                if (definition === undefined) return;

                definition.secrets = definition.secrets.map((s) =>
                  s.id === id ? response : s,
                );
                this.secretIsTransitioning[id] = false;
              },
            });
          },
          async SAVE_SECRET(
            definitionId: SecretDefinitionId,
            name: string,
            value: Record<string, string>,
            description?: string,
          ) {
            if (_.isEmpty(name)) {
              throw new Error("All secrets must have a name.");
            }

            if (this.secretsByDefinitionId[definitionId] === undefined) {
              throw new Error(
                "All secrets must be created based on a definition.",
              );
            }

            const user = useAuthStore().user;

            if (_.isNil(user)) {
              throw new Error(
                "All secrets must be created by a specific user.",
              );
            }

            const tempId = `-${Math.floor(
              Math.random() * 899999 + 100000,
            ).toString()}`;

            // Encrypt Value
            if (_.isNil(this.publicKey)) {
              throw new Error("Couldn't fetch publicKey.");
            }

            const algorithm = SecretAlgorithm.Sealedbox;
            const version = SecretVersion.V1;

            const keyPairPk = this.publicKey.pk;

            const crypted = await encryptMessage(value, this.publicKey);

            return new ApiRequest<Secret>({
              method: "post",
              url: "secret",
              params: {
                ...visibilityParams,
                name,
                description,
                definition: definitionId,
                crypted,
                keyPairPk,
                version,
                algorithm,
              },
              optimistic: () => {
                const { pk: userId, name: userName } = user;

                this.secretDefinitionByDefinitionId[
                  definitionId
                ]?.secrets?.push({
                  id: tempId,
                  definition: definitionId,
                  name,
                  description,
                  createdInfo: {
                    actor: { kind: "user", label: userName, id: userId },
                    timestamp: Date(),
                  },
                });
                this.secretIsTransitioning[tempId] = true;

                return () => {
                  const definition =
                    this.secretDefinitionByDefinitionId[definitionId];

                  if (definition === undefined) return;

                  definition.secrets = definition.secrets.filter(
                    (s) => s.id !== tempId,
                  );
                  this.secretIsTransitioning[tempId] = false;
                };
              },
              onSuccess: (response) => {
                const definition =
                  this.secretDefinitionByDefinitionId[definitionId];

                if (definition === undefined) return;

                definition.secrets = definition.secrets.map((s) =>
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
          async GET_PUBLIC_KEY() {
            return new ApiRequest<PublicKey>({
              url: "secret/get_public_key",
              params: {
                ...visibilityParams,
              },
              onSuccess: (response) => {
                this.publicKey = response;
              },
            });
          },
        },
        onActivated() {
          // TODO Run load secrets on websocket message too
          this.LOAD_SECRETS();
          this.GET_PUBLIC_KEY();

          const realtimeStore = useRealtimeStore();
          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            {
              eventType: "ChangeSetWritten",
              debounce: true,
              callback: (writtenChangeSetId) => {
                // ideally we wouldn't have to check this - since the topic subscription
                // would mean we only receive the event for this changeset already...
                // but this is fine for now
                if (writtenChangeSetId !== changeSetId) return;

                // probably want to get pushed updates instead of blindly re-fetching, but this is the first step of getting things working
                this.LOAD_SECRETS();
              },
            },
          ]);
        },
      },
    ),
  )();
}
