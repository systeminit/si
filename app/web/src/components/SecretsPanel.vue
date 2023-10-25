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
      <div
        ref="addingSecretNameRef"
        v-tooltip="addingSecretTooltip"
        class="text-center text-sm italic text-neutral-400 break-words line-clamp-3"
      >
        Defintion: "{{ addingSecretId }}"
      </div>
    </div>
    <AddSecretForm
      :definitionId="addingSecretId"
      :editingSecret="editingSecret"
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
      useDifferentLabelWhenOpen
    >
      <template #label>
        <div class="flex-grow text-md font-bold truncate">
          {{ definition.id }}
        </div>
      </template>
      <template #openLabel>
        <div class="flex-grow text-md font-bold break-words overflow-hidden">
          {{ definition.id }}
        </div>
      </template>
      <template #right>
        <div class="flex flex-row flex-none items-center gap-xs pl-xs">
          <PillCounter
            :count="secretsStore.secretsByDefinitionId[definition.id]?.length"
            showIfZero
            size="md"
            class="min-w-[27.1px] text-center"
          />
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
          @edit="openAddSecretForm(definition.id, secret)"
        />
      </template>
    </Collapsible>
  </ScrollArea>
</template>

<script lang="ts" setup>
import {
  Collapsible,
  PillCounter,
  ScrollArea,
  VButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import clsx from "clsx";
import { Secret, SecretId, useSecretsStore } from "@/store/secrets.store";
import SecretCard from "./SecretCard.vue";
import AddSecretForm from "./AddSecretForm.vue";

const secretsStore = useSecretsStore();

const addingSecretId = ref();
const editingSecret = ref();

const openAddSecretForm = (secretId: SecretId, edit?: Secret) => {
  editingSecret.value = edit;
  addingSecretId.value = secretId;
};

const closeAddSecretForm = () => {
  addingSecretId.value = undefined;
};

const addingSecretNameRef = ref();
const addingSecretTooltip = computed(() => {
  if (
    addingSecretNameRef.value &&
    addingSecretNameRef.value.scrollHeight >
      addingSecretNameRef.value.offsetHeight
  ) {
    return {
      content: addingSecretId,
      delay: { show: 700, hide: 10 },
    };
  } else {
    return {};
  }
});
</script>
