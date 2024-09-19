<template>
  <div class="w-full h-full grow flex flex-col overflow-hidden">
    <div ref="mainDivRef" class="overflow-y-auto flex flex-col">
      <div class="p-xs flex flex-col">
        <VormInput
          v-model="secretFormData.name"
          type="text"
          label="Display Name"
          compact
          compactWide
          required
          placeholder="The display name in System Initiative"
        />
        <VormInput
          v-model="secretFormData.description"
          type="textarea"
          label="Description"
          compact
          compactWide
          placeholder="A description for your reference"
        />
      </div>
      <div
        :class="
          clsx(
            'text-xs p-xs italic border-y',
            themeClasses(
              'border-neutral-200 text-neutral-500',
              'border-neutral-700 text-neutral-400',
            ),
          )
        "
      >
        Fields in the section below will be encrypted
      </div>
      <div class="relative">
        <template v-if="editingSecret && !replacingEnabled">
          <div
            :class="
              clsx(
                'absolute w-full h-full z-50 opacity-80',
                themeClasses('bg-caution-lines-light', 'bg-caution-lines-dark'),
              )
            "
          ></div>
          <div
            class="absolute w-full h-full z-60 flex flex-col items-center justify-center gap-xs"
          >
            <div
              class="mx-sm px-xs py-2xs text-center text-sm font-bold dark:bg-shade-100 bg-shade-0 rounded"
            >
              You cannot edit the encrypted data stored in this secret, but you
              can replace it with new data.
            </div>
            <VButton
              size="sm"
              label="Replace Secret"
              @click="enableReplacing"
            />
          </div>
        </template>
        <div
          :class="
            clsx(
              'w-full flex flex-col p-xs',
              editingSecret &&
                !replacingEnabled &&
                'min-h-[100px] justify-center',
            )
          "
        >
          <VormInput
            v-for="(field, index) in fields"
            :key="index"
            v-model="secretFormData.value[field.name]"
            :type="fieldInputType(field)"
            :label="field.name"
            compact
            compactWide
          />
          <!--:required="!editingSecret || replacingSecret"-->
        </div>
      </div>
    </div>
    <ErrorMessage :requestStatus="addSecretReqStatus" />
    <div
      :class="
        clsx(
          'flex-none w-full flex flex-row p-xs gap-sm bg-shade-0',
          mainDivScrolling ? 'dark:bg-shade-100' : 'dark:bg-neutral-800',
        )
      "
    >
      <VButton
        v-if="!hideCancelButton && !addSecretReqStatus.isPending"
        label="Cancel"
        tone="destructive"
        variant="ghost"
        @click="emit('cancel')"
      />
      <VButton
        v-if="editingSecret"
        class="grow"
        tone="action"
        loadingText="Updating Secret..."
        successText="Secret Updated!"
        label="Update Secret"
        :requestStatus="editSecretReqStatus"
        :disabled="validationState.isError"
        @click="updateSecret"
      />
      <VButton
        v-else
        class="grow"
        tone="action"
        loadingText="Storing Secret..."
        successText="Secret Stored!"
        label="Store Secret"
        :requestStatus="addSecretReqStatus"
        :disabled="validationState.isError"
        @click="saveSecret"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  VormInput,
  VButton,
  useValidatedInputGroup,
  ErrorMessage,
  themeClasses,
} from "@si/vue-lib/design-system";
import { PropType, ref, computed, onMounted, onBeforeUnmount } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import {
  Secret,
  SecretDefinitionId,
  SecretFormSchema,
  useSecretsStore,
} from "@/store/secrets.store";

const mainDivRef = ref();
const mainDivScrolling = ref(false);

const windowResizeHandler = () => {
  if (!mainDivRef.value) return;
  const el = mainDivRef.value;
  mainDivScrolling.value = el.scrollHeight > el.clientHeight;
};
onMounted(() => {
  window.addEventListener("resize", windowResizeHandler);
  windowResizeHandler();
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
});

const { validationState, validationMethods } = useValidatedInputGroup();

const props = defineProps({
  definitionId: {
    type: String as PropType<SecretDefinitionId>,
    required: true,
  },
  editingSecret: {
    type: Object as PropType<Secret>,
  },
  hideCancelButton: {
    type: Boolean,
  },
  replacing: { type: Boolean },
});

const secretsStore = useSecretsStore();

const fields = computed(
  () => secretsStore.secretFormSchemaByDefinitionId[props.definitionId],
);

const addSecretReqStatus = secretsStore.getRequestStatus("SAVE_SECRET");
const editSecretReqStatus = secretsStore.getRequestStatus("UPDATE_SECRET");

const secretFormEmpty = {
  name: "",
  description: "",
  value: {} as Record<string, string>,
};

const secretFormData = ref(
  props.editingSecret
    ? {
        name: props.editingSecret.name,
        description: props.editingSecret.description,
        value: {} as Record<string, string>,
      }
    : _.cloneDeep(secretFormEmpty),
);

const saveSecret = async () => {
  if (validationMethods.hasError()) return;

  const res = await secretsStore.SAVE_SECRET(
    props.definitionId,
    secretFormData.value.name,
    secretFormData.value.value,
    secretFormData.value.description,
  );

  if (res.result.success) {
    const secret = res.result.data;
    secretsStore.clearRequestStatus("SAVE_SECRET");
    secretFormData.value = _.cloneDeep(secretFormEmpty);
    validationMethods.resetAll();
    emit("save", secret);
  }
};

const replacingSecret = ref(false);
const enableReplacing = () => {
  replacingSecret.value = true;
};
const replacingEnabled = computed(
  () => replacingSecret.value || props.replacing,
);

const updateSecret = async () => {
  if (validationMethods.hasError() || !props.editingSecret) return;

  const secret = _.cloneDeep(props.editingSecret);
  secret.name = secretFormData.value.name;
  secret.description = secretFormData.value.description;

  const res = await secretsStore.UPDATE_SECRET(
    secret,
    _.isEmpty(secretFormData.value.value)
      ? undefined
      : secretFormData.value.value,
  );

  if (res.result.success) {
    const secret = res.result.data;
    secretsStore.clearRequestStatus("UPDATE_SECRET");
    secretFormData.value = _.cloneDeep(secretFormEmpty);
    validationMethods.resetAll();
    emit("save", secret);
  }
};

const fieldInputType = (field: SecretFormSchema) => {
  if (field.widgetKind.kind === "password") {
    return "password";
  } else if (field.widgetKind.kind === "textArea") {
    return "textarea";
  } else {
    return "text";
  }
};

const emit = defineEmits<{
  (e: "cancel"): void;
  (e: "save", v: Secret): void;
}>();
</script>
