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
          Secret: {{ definitionId }}
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

    <AddSecretForm v-if="addingSecret" definitionId="definitionId" />
    <div v-else class="overflow-y-auto flex flex-col h-full">
      <template v-if="secrets.length > 0">
        <SecretCard
          v-for="secret in secrets"
          :key="secret.id"
          :secret="secret"
        />
      </template>
      <div v-else class="flex flex-row items-center grow">
        <div class="text-center w-full">
          No secrets of this definition found.
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { VButton } from "@si/vue-lib/design-system";
import { ref, computed } from "vue";
import clsx from "clsx";
import { useSecretsStore, SecretDefinitionId } from "@/store/secrets.store";
import SecretCard from "./SecretCard.vue";
import AddSecretForm from "./AddSecretForm.vue";

const props = defineProps<{ definitionId: SecretDefinitionId }>();

const secretsStore = useSecretsStore();

const secrets = computed(
  () => secretsStore.secretsByDefinitionId[props.definitionId] ?? [],
);

const addingSecret = ref(false);

const showAddSecretForm = () => {
  addingSecret.value = true;
};

const cancelAddSecretForm = () => {
  addingSecret.value = false;
};
</script>
