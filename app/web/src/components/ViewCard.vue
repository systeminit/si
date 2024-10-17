<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center text-sm relative p-2xs pl-xs min-w-0 w-full border border-transparent cursor-pointer',
        selected
          ? 'dark:bg-action-900 bg-action-100 border-action-500 dark:border-action-300'
          : 'dark:border-neutral-700',
      )
    "
  >
    <div class="flex flex-col flex-grow min-w-0">
      <TruncateWithTooltip class="w-full">
        <span class="text-sm">
          {{ view.name }}
        </span>
      </TruncateWithTooltip>
    </div>
    <DropdownMenu ref="contextMenuRef" :forceAbove="false" forceAlignRight>
      <DropdownMenuItem
        :onSelect="
          () => {
            modalRef?.open();
          }
        "
        label="Edit Name"
      />
      <DropdownMenuItem disabled label="Delete View" />
    </DropdownMenu>
    <DetailsPanelMenuIcon
      :selected="contextMenuRef?.isOpen"
      @click="
        (e) => {
          contextMenuRef?.open(e, false);
        }
      "
    />
    <Modal
      ref="modalRef"
      type="save"
      size="sm"
      saveLabel="Save"
      title="Update View Name"
      @save="updateName"
    >
      <VormInput
        ref="labelRef"
        v-model="viewName"
        required
        label="View Name"
        @enterPressed="updateName"
      />
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";

import {
  Modal,
  VormInput,
  DropdownMenu,
  DropdownMenuItem,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ViewDescription } from "@/api/sdf/dal/views";
import { useViewsStore } from "@/store/views.store";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const viewStore = useViewsStore();

const props = defineProps<{
  selected?: boolean;
  view: ViewDescription;
}>();

// const confirmRef = ref<InstanceType<typeof ConfirmHoldModal> | null>(null);

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();
const modalRef = ref<InstanceType<typeof Modal>>();
const labelRef = ref<InstanceType<typeof VormInput>>();

const viewName = ref("");

const updateName = (e?: Event) => {
  e?.preventDefault();
  if (!viewName.value) {
    labelRef.value?.setError("Name is required");
  } else {
    viewStore.UPDATE_VIEW_NAME(props.view.id, viewName.value);
    modalRef.value?.close();
    viewName.value = "";
  }
};

watch(
  props.view,
  () => {
    viewName.value = props.view.name;
  },
  { immediate: true },
);

const emit = defineEmits<{
  (e: "closeDrawer"): void;
}>();
</script>
