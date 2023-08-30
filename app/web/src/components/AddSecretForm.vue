<template>
  <div class="overflow-y-auto flex flex-col gap-sm w-full h-full p-sm">
    <!--  TODO: Add form validation  -->
    <VormInput v-model="secretFormData.name" type="text" label="Name">
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
    />
    <VButton
      size="sm"
      tone="action"
      loadingText="Storing Secret"
      label="Store Secret"
      @click="saveSecret"
    />
  </div>
</template>

<script setup lang="ts">
import { VormInput, VButton } from "@si/vue-lib/design-system";
import { reactive } from "vue";
import { SecretDefinitionId, useSecretsStore } from "@/store/secrets.store";

const props = defineProps<{ definitionId: SecretDefinitionId }>();

const secretsStore = useSecretsStore();

const secretFormData = reactive({
  name: "",
  description: "",
});

const saveSecret = () => {
  secretsStore.SAVE_SECRET(
    props.definitionId,
    secretFormData.name,
    {},
    secretFormData.description,
  );
};
</script>
