<template>
  <div class="w-full h-full flex flex-col overflow-hidden">
    <div
      v-if="addSecretReqStatus.isSuccess"
      class="grow flex flex-row items-center"
    >
      <div class="w-full text-center text-2xl font-bold">Secret Stored!</div>
    </div>
    <template v-else>
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
        <VormInput
          v-for="field in testDefinition.fields"
          :key="field.id"
          v-model="secretFormData.value[field.id]"
          type="text"
          :label="field.displayName"
          required
        />
      </div>
      <ErrorMessage :requestStatus="addSecretReqStatus" />
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
          tone="action"
          loadingText="Storing Secret..."
          successText="Secret Stored!"
          label="Store Secret"
          :requestStatus="addSecretReqStatus"
          :disabled="validationState.isError"
          @click="saveSecret"
        />
        <VButton
          v-if="!hideCancelButton && !addSecretReqStatus.isPending"
          label="Cancel"
          tone="destructive"
          variant="ghost"
          @click="emit('cancel')"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import {
  VormInput,
  VButton,
  useValidatedInputGroup,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { PropType, reactive } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import {
  Secret,
  SecretDefinition,
  SecretDefinitionId,
  useSecretsStore,
} from "@/store/secrets.store";

// TODO(Wendy) - replace this test definition with a lookup of the given definitionId's definition
const testDefinition: SecretDefinition = {
  fields: {
    test1: { id: "test1", displayName: "Test Value", value: "" },
    test2: { id: "test2", displayName: "Whatever", value: "" },
  },
};

const { validationState, validationMethods } = useValidatedInputGroup();

const props = defineProps({
  definitionId: {
    type: String as PropType<SecretDefinitionId>,
    required: true,
  },
  hideCancelButton: {
    type: Boolean,
  },
});

const secretsStore = useSecretsStore();

const addSecretReqStatus = secretsStore.getRequestStatus("SAVE_SECRET");

const secretFormEmpty = {
  name: "",
  description: "",
  value: {} as Record<string, string>,
  expiration: "",
};

const secretFormData = reactive(_.clone(secretFormEmpty));

const saveSecret = async () => {
  if (validationMethods.hasError()) return;

  const res = await secretsStore.SAVE_SECRET(
    props.definitionId,
    secretFormData.name,
    secretFormData.value,
    secretFormData.description,
    secretFormData.expiration,
  );

  if (res.result.success) {
    const secret = res.result.data;
    setTimeout(() => {
      secretsStore.clearRequestStatus("SAVE_SECRET");

      _.assign(secretFormData, secretFormEmpty);

      validationMethods.resetAll();

      emit("save", secret);
    }, 2000);
  }
};

const emit = defineEmits<{
  (e: "cancel"): void;
  (e: "save", v: Secret): void;
}>();
</script>
