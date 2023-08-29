<template>
  <div
    :class="
      clsx(
        'bg-neutral-100 dark:bg-neutral-700 w-96 h-96 rounded flex flex-col overflow-hidden text-white shadow-3xl',
      )
    "
  >
    <div
      class="bg-shade-0 dark:bg-shade-100 p-xs shrink-0 flex flex-row justify-between"
    >
      <div class="flex flex-col">
        <div
          class="uppercase font-bold text-md pb-xs text-shade-100 dark:text-shade-0"
        >
          Secret: {{ definitionName }}
        </div>
        <div class="text-xs italic text-neutral-600 dark:text-neutral-500">
          <template v-if="addingSecret">
            Fill out the form below to add the secret.
          </template>
          <template v-else>
            Select a secret from the list or add a new one.
          </template>
        </div>
      </div>
      <VButton
        v-if="addingSecret"
        icon="x"
        tone="destructive"
        @click="cancelAddSecretForm"
      />
      <VButton
        v-else
        label="Add"
        icon="plus"
        tone="action"
        @click="showAddSecretForm"
      />
    </div>

    <AddSecretForm v-if="addingSecret" />
    <div v-else class="overflow-y-auto flex flex-col h-full">
      <template v-if="mockData.length > 0">
        <SecretCard
          v-for="secret in mockData"
          :key="secret.id"
          :secret="secret"
        />
      </template>
      <div v-else class="flex flex-row items-center grow">
        <div class="text-center w-full">
          No secrets of this defintion found.
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { VButton } from "@si/vue-lib/design-system";
import { ref } from "vue";
import clsx from "clsx";
import { Secret } from "@/store/secrets.store";
import { ActorAndTimestamp } from "@/store/components.store";
import SecretCard from "./SecretCard.vue";
import AddSecretForm from "./AddSecretForm.vue";

const props = defineProps({
  definitionName: { type: String, required: true },
});

const addingSecret = ref(false);

const showAddSecretForm = () => {
  addingSecret.value = true;
};

const cancelAddSecretForm = () => {
  addingSecret.value = false;
};

const mockData = [
  {
    id: "mock secret id 1",
    definition: props.definitionName,
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
    definition: props.definitionName,
    name: "Mock Secret Name 2 here this name is very long omg testing long names is important!",
    description: "this is a shorter description",
    createdInfo: {
      actor: { kind: "user", label: "cooldood420" },
      timestamp: new Date("12/20/2021").toDateString(),
    } as ActorAndTimestamp,
  },
  {
    id: "mock secret id 3",
    definition: props.definitionName,
    name: "Mock Secret Name 3",
    description: "",
    createdInfo: {
      actor: {
        kind: "user",
        label: "whateverpersonlongusernamewowthatisreallylongidkwaytoolong",
      },
      timestamp: new Date("01/01/2023").toDateString(),
    } as ActorAndTimestamp,
  },
  {
    id: "mock secret id 4",
    definition: props.definitionName,
    name: "Mock Secret Name 4",
    description: "this one is cool",
    createdInfo: {
      actor: { kind: "user", label: "angiecat" },
      timestamp: new Date().toDateString(),
    } as ActorAndTimestamp,
  },
  {
    id: "mock secret id 5",
    definition: props.definitionName,
    name: "Mock Secret Name 5",
    description: "",
    createdInfo: {
      actor: { kind: "user", label: "gabycat" },
      timestamp: new Date().toDateString(),
    } as ActorAndTimestamp,
  },
  {
    id: "mock secret id 6",
    definition: props.definitionName,
    name: "THE FINAL MOCK SECRET",
    description:
      "with a description that fits on two lines but is not long enough to be truncated at all",
    createdInfo: {
      actor: { kind: "system", label: "System Initiative" },
      timestamp: new Date().toDateString(),
    } as ActorAndTimestamp,
  },
] as Secret[];
</script>
