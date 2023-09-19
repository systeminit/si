import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";

import { useAuthStore } from "@/store/auth.store";
import { useChangeSetsStore, ChangeSetId } from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
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

export type SecretsHashMap = Record<SecretDefinitionId, Secret[]>;

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
        secretsByDefinitionId: {} as SecretsHashMap,
        secretIsTransitioning: {} as Record<SecretId, boolean>,
      }),
      getters: {
        secrets: (state) => _.flatMap(state.secretsByDefinitionId),
        secretsById(): Record<SecretId, Secret> {
          return _.keyBy(this.secrets, (s) => s.id);
        },
        definitions: (state) => _.keys(state.secretsByDefinitionId),
      },
      actions: {
        async LOAD_SECRETS() {
          return new ApiRequest<SecretsHashMap>({
            url: "secret",
            params: {
              ...visibilityParams,
            },
            onSuccess: (response) => {
              this.secretsByDefinitionId = response;
              if (!this.secretsByDefinitionId.Mocks)
                this.secretsByDefinitionId.Mocks = [];

              // this.secretsByDefinitionId.Mocks = [
              //   {
              //     id: "mock secret id 1",
              //     definition: "Mocks",
              //     name: "Mock Secret Name 1",
              //     description:
              //       "this is the description of the secret written by the user it can be very long and they can just put as much content as they want Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa  qui officia deserunt mollit anim id est laborum",
              //     createdInfo: {
              //       actor: { kind: "user", label: "wendywildshape" },
              //       timestamp: new Date().toDateString(),
              //     } as ActorAndTimestamp,
              //   },
              //   {
              //     id: "mock secret id 2",
              //     definition: "Mocks",
              //     name: "Mock Secret Name 2 here this name is very long omg testing long names is important!",
              //     description: "this is a shorter description",
              //     createdInfo: {
              //       actor: { kind: "user", label: "cooldood420" },
              //       timestamp: new Date("12/20/2021").toDateString(),
              //     } as ActorAndTimestamp,
              //   },
              //   {
              //     id: "mock secret id 3",
              //     definition: "Mocks",
              //     name: "Mock Secret Name 3",
              //     description: "",
              //     createdInfo: {
              //       actor: {
              //         kind: "user",
              //         label:
              //           "whateverpersonlongusernamewowthatisreallylongidkwaytoolong",
              //       },
              //       timestamp: new Date("01/01/2023").toDateString(),
              //     } as ActorAndTimestamp,
              //   },
              //   {
              //     id: "mock secret id 4",
              //     definition: "Mocks",
              //     name: "Mock Secret Name 4",
              //     description: "this one is cool",
              //     createdInfo: {
              //       actor: { kind: "user", label: "angiecat" },
              //       timestamp: new Date().toDateString(),
              //     } as ActorAndTimestamp,
              //   },
              //   {
              //     id: "mock secret id 5",
              //     definition: "Mocks",
              //     name: "Mock Secret Name 5",
              //     description: "",
              //     createdInfo: {
              //       actor: { kind: "user", label: "gabycat" },
              //       timestamp: new Date().toDateString(),
              //     } as ActorAndTimestamp,
              //   },
              //   {
              //     id: "mock secret id 6",
              //     definition: "Mocks",
              //     name: "THE FINAL MOCK SECRET",
              //     description:
              //       "with a description that fits on two lines but is not long enough to be truncated at all",
              //     createdInfo: {
              //       actor: { kind: "system", label: "System Initiative" },
              //       timestamp: new Date().toDateString(),
              //     } as ActorAndTimestamp,
              //   },
              // ];
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
