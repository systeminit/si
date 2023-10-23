<template>
  <div
    v-if="addingSecretId"
    class="w-full h-full flex flex-col overflow-hidden"
  >
    <div
      class="p-xs flex-none flex flex-col border-b border-neutral-200 dark:border-neutral-600"
    >
      <div class="text-lg font-bold text-center">
        {{ editingSecret ? "Editing Secret" : "New Secret" }}
      </div>
      <div class="text-center text-sm italic text-neutral-400">
        Defintion: "{{ addingSecretId }}"
      </div>
    </div>
    <AddSecretForm
      :definitionId="addingSecretId"
      @save="closeAddSecretForm"
      @cancel="closeAddSecretForm"
    />
  </div>
  <ScrollArea
    v-else
    :class="
      clsx(
        'flex flex-col w-full h-full',
        addingSecretId && 'justify-items-stretch',
      )
    "
  >
    <template #top>
      <div
        class="p-xs text-lg font-bold text-center border-b border-neutral-200 dark:border-neutral-600"
      >
        Secret Defintions
      </div>
    </template>
    <Collapsible
      v-for="definition in secretsStore.secretsByLastCreated"
      :key="definition.id"
      buttonClasses="bg-neutral-100 dark:bg-neutral-900"
      :defaultOpen="false"
    >
      <template #label>
        <div class="flex-grow truncate text-lg font-bold">
          {{ definition.id }}
        </div>
      </template>
      <template #right>
        <div class="flex flex-row items-center gap-xs">
          <div
            :class="
              clsx(
                'text-md rounded-2xl px-xs border',
                themeClasses(
                  'border-neutral-600 text-neutral-600 bg-neutral-200',
                  'border-neutral-300 text-neutral-300 bg-neutral-700',
                ),
              )
            "
          >
            {{ secretsStore.secretsByDefinitionId[definition.id]?.length }}
          </div>
          <VButton
            icon="plus"
            tone="action"
            size="xs"
            rounded
            @click.stop="openAddSecretForm(definition.id)"
          />
        </div>
      </template>
      <template #default>
        <div
          v-if="secretsStore.secretsByDefinitionId[definition.id]?.length === 0"
          class="p-sm text-center text-neutral-400 border-neutral-200 dark:border-neutral-500 border-b"
        >
          No secrets of this definition found.
        </div>
        <SecretCard
          v-for="secret in secretsStore.secretsByDefinitionId[definition.id]"
          v-else
          :key="secret.id"
          :secret="secret"
          detailedListItem
        />
      </template>
    </Collapsible>
  </ScrollArea>
</template>

<script lang="ts" setup>
import {
  Collapsible,
  ScrollArea,
  VButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import { ref } from "vue";
import clsx from "clsx";
import { SecretId, useSecretsStore } from "@/store/secrets.store";
import SecretCard from "./SecretCard.vue";
import AddSecretForm from "./AddSecretForm.vue";

const secretsStore = useSecretsStore();

const test = secretsStore.secretsByLastCreated;

const addingSecretId = ref();
const editingSecret = ref();

const openAddSecretForm = (secretId: SecretId) => {
  addingSecretId.value = secretId;
};

const closeAddSecretForm = () => {
  addingSecretId.value = undefined;
};
</script>
