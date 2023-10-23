<template>
  <div
    :class="
      clsx('w-full h-full flex flex-col overflow-hidden', themeContainerClasses)
    "
  >
    <div
      v-if="addSecretReqStatus.isSuccess"
      class="grow flex flex-row items-center"
    >
      <div class="w-full text-center text-2xl font-bold">
        <template v-if="editingSecret">Secret Updated!</template>
        <template v-else>Secret Stored!</template>
      </div>
    </div>
    <template v-else>
      <div ref="mainDivRef" class="overflow-y-auto flex flex-col">
        <div class="p-sm flex flex-col gap-sm">
          <!--  TODO: Add form validation  -->
          <VormInput
            v-model="secretFormData.name"
            type="text"
            label="Name"
            required
          >
            <template #instructions>
              <div
                :class="
                  clsx(
                    'italic',
                    themeClasses('text-neutral-700', 'text-neutral-400'),
                  )
                "
              >
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
              <div
                :class="
                  clsx(
                    'italic',
                    themeClasses('text-neutral-700', 'text-neutral-400'),
                  )
                "
              >
                Describe this secret in detail for your reference
              </div>
            </template>
          </VormInput>
        </div>
        <div
          :class="
            clsx(
              'border-t w-full p-sm flex flex-col gap-sm',
              themeClasses('border-neutral-200', 'border-neutral-600'),
            )
          "
        >
          <div
            :class="
              clsx(
                'text-sm italic',
                themeClasses('text-neutral-700', 'text-neutral-400'),
              )
            "
          >
            Fields in this section will be encrypted.
          </div>
          <!--VormInput
          v-model="secretFormData.expiration"
          type="date"
          label="Expiration"
        >
          <template #instructions>
            <div :class="
                  clsx(
                    'italic',
                    themeClasses('text-neutral-700', 'text-neutral-400'),
                  )
                ">
              Optional: Set an expiration date for this secret
            </div>
          </template>
        </VormInput-->
          <VormInput
            v-for="(field, index) in fields"
            :key="index"
            v-model="secretFormData.value[field.name]"
            type="text"
            :label="field.name"
            required
          />
        </div>
      </div>
      <ErrorMessage :requestStatus="addSecretReqStatus" />
      <div
        :class="
          clsx(
            'flex-none w-full flex flex-row p-xs gap-xs',
            forceDark ? 'bg-shade-100' : 'bg-shade-0',
            !forceDark &&
              (mainDivScrolling ? 'dark:bg-shade-100' : 'dark:bg-neutral-800'),
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
  useThemeContainer,
  themeClasses,
} from "@si/vue-lib/design-system";
import { PropType, ref, computed, onMounted, onBeforeUnmount } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import {
  Secret,
  SecretDefinitionId,
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
  forceDark: { type: Boolean },
});

const { themeContainerClasses } = useThemeContainer(
  props.forceDark ? "dark" : undefined,
);

const secretsStore = useSecretsStore();

const fields = computed(
  () => secretsStore.secretFormSchemaByDefinitionId[props.definitionId],
);

const addSecretReqStatus = secretsStore.getRequestStatus("SAVE_SECRET");

const secretFormEmpty = {
  name: "",
  description: "",
  value: {} as Record<string, string>,
};

const secretFormData = ref(_.cloneDeep(secretFormEmpty));

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
    setTimeout(() => {
      secretsStore.clearRequestStatus("SAVE_SECRET");

      secretFormData.value = _.cloneDeep(secretFormEmpty);

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
