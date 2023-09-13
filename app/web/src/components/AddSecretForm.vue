<template>
  <div class="w-full h-full flex flex-col overflow-hidden">
    <div class="overflow-y-auto flex flex-col gap-sm p-sm">
      <!--  TODO: Add form validation  -->
      <VormInput
        v-model="secretFormData.name"
        type="text"
        label="Name"
        required
      >
        <template #instructions>
          <div class="text-neutral-700 dark:text-neutral-400 italic">
            The display name for this secret within System Initiative
          </div>
        </template>
      </VormInput>
      <VormInput
        v-model="secretFormData.description"
        type="textarea"
        label="Description"
      >
        <template #instructions>
          <div class="text-neutral-700 dark:text-neutral-400 italic">
            Describe this secret in detail for your reference
          </div>
        </template>
      </VormInput>
      <VormInput
        v-model="secretFormData.expiration"
        type="date"
        label="Expiration"
      >
        <template #instructions>
          <div class="text-neutral-700 dark:text-neutral-400 italic">
            Optional: Set an expiration date for this secret
          </div>
        </template>
      </VormInput>
      <!-- TODO - this form should generate this part based on the secret definition, for now it's just a Value field -->
      <VormInput
        v-model="secretFormData.value"
        type="textarea"
        label="Value"
        required
      />
    </div>
    <div
      :class="
        clsx(
          'flex-none w-full flex flex-row p-xs gap-xs',
          // 'bg-shade-0 dark:bg-shade-100', // dark/light mode classes
          'bg-shade-100', // force dark mode class
        )
      "
    >
      <VButton
        class="grow"
        size="sm"
        tone="action"
        loadingText="Storing Secret"
        label="Store Secret"
        @click="saveSecret"
      />
      <VButton
        label="Cancel"
        tone="destructive"
        variant="ghost"
        @click="emit('cancel')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { VormInput, VButton } from "@si/vue-lib/design-system";
import { reactive } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import { SecretDefinitionId, useSecretsStore } from "@/store/secrets.store";

const props = defineProps<{ definitionId: SecretDefinitionId }>();

const secretsStore = useSecretsStore();

const secretFormData = reactive({
  name: "",
  description: "",
  value: "",
  expiration: "",
});

const saveSecret = () => {
  secretsStore.SAVE_SECRET(
    props.definitionId,
    secretFormData.name,
    {},
    secretFormData.description,
    secretFormData.expiration,
  );

  secretFormData.name = "";
  secretFormData.description = "";
  secretFormData.value = "";
  secretFormData.expiration = "";
};

const emit = defineEmits<{
  (e: "cancel"): void;
}>();
</script>
