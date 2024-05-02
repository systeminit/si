<template>
  <div
    v-if="addingSecretId"
    class="w-full h-full flex flex-col overflow-hidden"
  >
    <SecretsPanelTitle
      :title="editingSecret ? 'Editing Secret' : 'New Secret'"
      :subtitle="`Defintion: ${addingSecretId}`"
      :subtitleTooltip="addingSecretTooltip"
    />
    <AddSecretForm
      :definitionId="addingSecretId"
      :editingSecret="editingSecret"
      @save="completeAddSecretForm"
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
      <SecretsPanelTitle title="Secret Definitions" />
    </template>

    <TreeNode
      v-for="definition in secretsStore.secretsByLastCreated"
      :key="definition.id"
      alwaysShowArrow
      enableGroupToggle
      :defaultOpen="false"
      classes="bg-neutral-100 dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-600"
      noIndentationOrLeftBorder
      enableDefaultHoverClasses
    >
      <template #label>
        <div class="flex-grow text-md font-bold truncate leading-loose">
          {{ definition.id }}
        </div>
      </template>
      <template #openLabel>
        <div class="flex-grow text-md font-bold break-words overflow-hidden">
          {{ definition.id }}
        </div>
      </template>
      <template #icons>
        <div class="flex flex-row flex-none items-center gap-xs pl-xs">
          <PillCounter
            :count="secretsStore.secretsByDefinitionId[definition.id]?.length"
            showHoverInsideTreeNode
            size="md"
            class="min-w-[27.1px] text-center py-[1px] font-bold"
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
    </TreeNode>

    <div
      v-if="secretsStore.secretsByLastCreated.length === 0"
      class="w-full text-center p-sm text-neutral-500 dark:text-neutral-400 italic"
    >
      No secret definitions found.
    </div>
  </ScrollArea>
</template>

<script lang="ts" setup>
import {
  PillCounter,
  ScrollArea,
  TreeNode,
  VButton,
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import clsx from "clsx";
import { Secret, SecretId, useSecretsStore } from "@/store/secrets.store";
import SecretCard from "./SecretCard.vue";
import AddSecretForm from "./AddSecretForm.vue";
import SecretsPanelTitle from "./SecretsPanelTitle.vue";

const secretsStore = useSecretsStore();

const addingSecretId = ref();
const editingSecret = ref();
const openDefinitionOnLoad = ref();

const openAddSecretForm = (secretId: SecretId, edit?: Secret) => {
  editingSecret.value = edit;
  addingSecretId.value = secretId;
};

const closeAddSecretForm = () => {
  addingSecretId.value = undefined;
};

const completeAddSecretForm = () => {
  openDefinitionOnLoad.value = addingSecretId.value;
  closeAddSecretForm();
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
