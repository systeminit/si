<template>
  <Modal ref="modalRef" size="sm" noInnerPadding noWrapper>
    <div
      :class="
        clsx(
          'flex flex-row gap-sm items-center',
          themeClasses('bg-white', 'bg-black'),
        )
      "
    >
      <Icon
        name="alert-circle"
        class="text-warning-600 content-center ml-md"
        size="lg"
      />
      <p class="grow py-md">
        This action has dependencies that will also be put on hold as a result
        of this action. Click <strong>OK</strong> to proceed...
      </p>
      <div class="flex flex-col self-stretch">
        <VButton
          label="OK"
          tone="empty"
          variant="solid"
          class="grow text-action-300 dark:hover:text-white hover:text-black hover:bg-action-400 hover:underline"
          @click="props.ok"
        ></VButton>
        <VButton
          class="grow text-action-300 dark:hover:text-white hover:text-black hover:bg-action-400 hover:underline"
          label="Cancel"
          tone="empty"
          variant="solid"
          @click="cancel"
        ></VButton>
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ref } from "vue";
import { VButton, Icon, Modal, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";

const modalRef = ref<InstanceType<typeof Modal> | null>(null);

const cancel = (): void => {
  if (props.cancel) props.cancel();
  else close();
};

const open = () => {
  modalRef.value?.open();
};

const close = () => {
  modalRef.value?.close();
};

const props = defineProps<{
  ok: () => void;
  cancel?: () => void;
}>();

defineExpose({ close, open });
</script>
