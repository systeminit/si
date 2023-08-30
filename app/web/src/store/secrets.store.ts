import { addStoreHooks } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import * as _ from "lodash-es";

import { useAuthStore } from "@/store/auth.store";
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
};

export function useSecretsStore() {
  return addStoreHooks(
    defineStore("secrets", {
      state: () => ({
        secrets: {} as Secret[],
      }),
      getters: {
        secretsByDefinitionId: (state) =>
          _.groupBy(state.secrets, (s) => s.definition),
        definitions(): SecretDefinitionId[] {
          return _.keys(this.secretsByDefinitionId);
        },
      },
      actions: {
        SAVE_SECRET(
          definition: SecretDefinitionId,
          name: string,
          value: Record<string, string>,
          description?: string,
        ) {
          const id = Math.floor(Math.random() * 899999 + 100000).toString();

          const { id: userId, name: userName } = useAuthStore().user ?? {
            id: "-1",
            name: "Anonymous",
          };

          this.secrets.push({
            id,
            definition,
            name,
            description,
            createdInfo: {
              actor: { kind: "user", label: userName, id: userId },
              timestamp: Date(),
            },
          });
        },
        DELETE_SECRET(id: SecretId) {
          this.secrets = this.secrets.filter((s) => s.id !== id);
        },
      },
      onActivated() {
        this.secrets = [
          {
            id: "001",
            definition: "AWS",
            name: "Production",
            createdInfo: {
              actor: { kind: "user", label: "SI Sally", id: "001" },
              timestamp: Date(),
            },
          },
          {
            id: "002",
            definition: "AWS",
            name: "Staging",
            createdInfo: {
              actor: { kind: "user", label: "SI Steve", id: "002" },
              timestamp: Date(),
            },
          },
          {
            id: "003",
            definition: "Azure",
            name: "Production",
            createdInfo: {
              actor: { kind: "user", label: "SI Sally", id: "001" },
              timestamp: Date(),
            },
          },
          {
            id: "mock secret id 1",
            definition: "Mocks",
            name: "Mock Secret Name 1",
            description:
              "this is the description of the secret written by the user it can be very long and they can just put as much content as they want Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa  qui officia deserunt mollit anim id est laborum",
            createdInfo: {
              actor: { kind: "user", label: "wendywildshape" },
              timestamp: new Date().toDateString(),
            } as ActorAndTimestamp,
          },
          {
            id: "mock secret id 2",
            definition: "Mocks",
            name: "Mock Secret Name 2 here this name is very long omg testing long names is important!",
            description: "this is a shorter description",
            createdInfo: {
              actor: { kind: "user", label: "cooldood420" },
              timestamp: new Date("12/20/2021").toDateString(),
            } as ActorAndTimestamp,
          },
          {
            id: "mock secret id 3",
            definition: "Mocks",
            name: "Mock Secret Name 3",
            description: "",
            createdInfo: {
              actor: {
                kind: "user",
                label:
                  "whateverpersonlongusernamewowthatisreallylongidkwaytoolong",
              },
              timestamp: new Date("01/01/2023").toDateString(),
            } as ActorAndTimestamp,
          },
          {
            id: "mock secret id 4",
            definition: "Mocks",
            name: "Mock Secret Name 4",
            description: "this one is cool",
            createdInfo: {
              actor: { kind: "user", label: "angiecat" },
              timestamp: new Date().toDateString(),
            } as ActorAndTimestamp,
          },
          {
            id: "mock secret id 5",
            definition: "Mocks",
            name: "Mock Secret Name 5",
            description: "",
            createdInfo: {
              actor: { kind: "user", label: "gabycat" },
              timestamp: new Date().toDateString(),
            } as ActorAndTimestamp,
          },
          {
            id: "mock secret id 6",
            definition: "Mocks",
            name: "THE FINAL MOCK SECRET",
            description:
              "with a description that fits on two lines but is not long enough to be truncated at all",
            createdInfo: {
              actor: { kind: "system", label: "System Initiative" },
              timestamp: new Date().toDateString(),
            } as ActorAndTimestamp,
          },
        ];
      },
    }),
  )();
}
