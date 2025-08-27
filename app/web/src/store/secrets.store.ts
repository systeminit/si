import { addStoreHooks } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";

import { useChangeSetsStore } from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { PropertyEditorPropWidgetKind } from "@/api/sdf/dal/property_editor";
import { ActorAndTimestamp } from "@/api/sdf/dal/component";
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
  isUsable: boolean;
  createdInfo: ActorAndTimestamp;
  updatedInfo?: ActorAndTimestamp;
  expiration?: string;
  connectedComponents: string[];
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

  const realtimeStore = useRealtimeStore();

  // this needs some work... but we'll probably want a way to force using HEAD
  // so we can load HEAD data in some scenarios while also loading a change set?
  const changeSetId = changeSetsStore.selectedChangeSetId;

  return addStoreHooks(
    workspaceId,
    changeSetId,
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
          registerRequestsBegin(requestUlid: string, actionName: string) {
            realtimeStore.inflightRequests.set(requestUlid, actionName);
          },
          registerRequestsEnd(requestUlid: string) {
            realtimeStore.inflightRequests.delete(requestUlid);
          },
        },
        onActivated() {
          return () => {
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
}
