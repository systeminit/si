<template>
  <div
    :class="
      clsx(
        themeContainerClasses,
        // 'bg-neutral-100 dark:bg-neutral-700 text-shade-100 dark:text-shade-0', // dark/light mode classes
        'bg-neutral-700 text-shade-0', // force dark mode classes
        'w-96 min-h-[24rem] max-h-[80vh] rounded flex flex-col overflow-hidden shadow-3xl',
      )
    "
  >
    <div
      :class="
        clsx(
          'p-xs shrink-0 flex flex-row gap-xs justify-between items-center',
          // 'bg-shade-0 dark:bg-shade-100 text-shade-100 dark:text-shade-0', // dark/light mode classes
          'bg-shade-100 text-shade-0', // force dark mode classes
        )
      "
    >
      <div class="flex flex-col overflow-hidden">
        <div class="uppercase font-bold text-md line-clamp-3 break-words">
          Secret: {{ definitionId }}
        </div>
        <div
          :class="
            clsx(
              'text-xs italic pt-xs',
              // 'text-neutral-600 dark:text-neutral-500', // dark/light mode classes
              'text-neutral-500', // force dark mode class
            )
          "
        >
          <template v-if="addingSecret">
            Fill out the form below to add the secret.
          </template>
          <template v-else>
            Select a secret from the list or add a new one.
          </template>
        </div>
      </div>
      <VButton
        v-if="!addingSecret"
        label="Add"
        icon="plus"
        tone="action"
        class="flex-none"
        @click="showAddSecretForm"
      />
      <!-- <VButton
        v-else
        icon="x"
        tone="destructive"
        @click="cancelAddSecretForm"
      /> -->
    </div>

    <AddSecretForm
      v-if="addingSecret"
      :definitionId="definitionId"
      forceDark
      @save="selectSecret"
      @cancel="cancelAddSecretForm"
    />
    <div v-else class="overflow-y-auto flex flex-col h-full flex-grow">
      <RequestStatusMessage
        v-if="loadSecretsReq.isPending"
        :requestStatus="loadSecretsReq"
        loadingMessage="Loading Secrets"
      />
      <template v-else-if="secrets.length > 0">
        <SecretCard
          v-for="secret in secrets"
          :key="secret.id"
          :secret="secret"
          @click="emit('select', secret)"
        />
      </template>
      <div v-else class="flex flex-row items-center grow">
        <div
          :class="
            clsx(
              'text-center w-full',
              // 'text-shade-100 dark:text-shade-0', // dark/light mode classes
              'text-shade-0', // force dark mode class
            )
          "
        >
          No secrets of this definition found.
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  VButton,
  RequestStatusMessage,
  useThemeContainer,
} from "@si/vue-lib/design-system";

import { ref, computed } from "vue";
import clsx from "clsx";
import {
  useSecretsStore,
  SecretDefinitionId,
  Secret,
} from "@/store/secrets.store";
import SecretCard from "./SecretCard.vue";
import AddSecretForm from "./AddSecretForm.vue";

const { themeContainerClasses } = useThemeContainer("dark");

const props = defineProps<{ definitionId: SecretDefinitionId }>();

const secretsStore = useSecretsStore();

const loadSecretsReq = secretsStore.getRequestStatus("LOAD_SECRETS");

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

const selectSecret = (secret: Secret) => {
  emit("select", secret);
};

const emit = defineEmits<{
  (e: "select", v: Secret): void;
}>();
</script>
