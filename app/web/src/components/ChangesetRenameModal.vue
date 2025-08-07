<template>
  <Modal
    ref="modal"
    title="Rename change set"
    size="sm"
    @click="inputRef?.focus()"
  >
    <div class="flex flex-col gap-sm">
      <VormInput
        ref="inputRef"
        v-model="changesetName"
        :disabled="loading"
        label="Name"
        noLabel
        required
        type="text"
        @enterPressed="submit"
      />
      <ErrorMessage
        v-if="apiErrorMessage"
        class="rounded-md text-md px-xs py-xs"
        icon="x-circle"
        variant="block"
      >
        <b>Error renaming change set:</b> <br />
        {{ apiErrorMessage }}
      </ErrorMessage>

      <div class="flex flex-row items-center justify-end w-full gap-xs">
        <VButton
          :loading="loading"
          size="xs"
          tone="neutral"
          @click="modal?.close()"
        >
          Cancel <TextPill tighter variant="key">Esc</TextPill>
        </VButton>
        <VButton :loading="loading" size="xs" @click="submit"> Rename </VButton>
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import { nextTick, ref } from "vue";
import {
  ErrorMessage,
  Modal,
  TextPill,
  VButton,
  VormInput,
} from "@si/vue-lib/design-system";
import {
  apiContextForChangeSet,
  routes,
  useApi,
} from "@/newhotness/api_composables";
import { useContext } from "@/newhotness/logic_composables/context";

const modal = ref<InstanceType<typeof Modal>>();
const inputRef = ref<InstanceType<typeof VormInput>>();

const loading = ref(false);
const changesetName = ref("");
const changesetIdRef = ref<string | undefined>();
const apiErrorMessage = ref<string | undefined>();

const ctx = useContext();

const submit = async () => {
  if (
    inputRef.value?.validationState.isError ||
    !changesetName.value ||
    !changesetIdRef.value
  )
    return;

  const apiCtx = apiContextForChangeSet(ctx, changesetIdRef.value);
  const renameChangesetApi = useApi(apiCtx);
  const call = renameChangesetApi.endpoint(routes.ChangeSetRename);

  loading.value = true;
  apiErrorMessage.value = undefined;

  const { req, errorMessage } = await call.post({
    newName: changesetName.value,
  });

  loading.value = false;
  if (req.status !== 200) {
    apiErrorMessage.value = errorMessage ?? "Unknown error";
    return;
  }

  modal.value?.close();
};

const open = (changesetId: string, initialValue?: string) => {
  loading.value = false;
  apiErrorMessage.value = undefined;
  changesetName.value = initialValue ?? "";
  changesetIdRef.value = changesetId;
  inputRef.value?.validationMethods.reset();
  modal.value?.open();

  nextTick(() => {
    inputRef.value?.focus();
  });
};

defineExpose({
  open,
  modal,
});

const emit = defineEmits(["submit"]);
</script>
