<template>
  <Modal ref="modalRef" title="Delete Component">
    <div class="max-h-[70vh] overflow-hidden flex flex-col">
      <div class="pb-xs">Are you sure you want to delete this component?</div>

      <!-- NOTE(nick): we should consider splitting this out into its own component... potentially to use with Map view too -->
      <div
        v-for="component in components"
        :key="component.id"
        :style="borderStyle(component)"
        class="flex items-center bg-neutral-700 rounded-sm border-l-2 p-xs space-x-sm"
      >
        <Icon :name="getAssetIcon(component.schemaCategory)" size="lg" />
        <div>
          <div class="text-sm font-semibold">{{ component.name }}</div>
          <div class="text-xs text-neutral-300">{{ component.schemaName }}</div>
        </div>
      </div>

      <div class="px-2xs py-xs">
        <VormInput v-model="mode" noLabel type="radio">
          <VormInputOption :value="DeleteMode.Delete">
            <div class="text-xs my-xs">
              <span class="flex flex-col">
                <strong>Delete</strong>
              </span>
              <span class="flex flex-col">
                If this component has a corresponding resource, and/or its
                outgoing dependencies do, it will be marked for deletion and
                removed when this change set is applied. Otherwise, the
                component will be deleted immediately.
              </span>
            </div>
          </VormInputOption>
          <VormInputOption :value="DeleteMode.Remove">
            <div class="text-xs my-xs">
              <span class="flex flex-col">
                <strong>Remove</strong>
              </span>
              <span class="flex flex-col">
                If this component exists in at least two views, it will be
                removed from the current view and will remain in other view(s).
                Otherwise, the remove request will "no-op" and the component
                will remain in the current view.
              </span>
            </div>
          </VormInputOption>
        </VormInput>
      </div>

      <div class="flex gap-sm">
        <VButton label="Cancel" tone="shade" variant="ghost" @click="close" />
        <VButton
          class="flex-grow"
          icon="trash"
          label="Confirm"
          tone="destructive"
          @click="onConfirm"
        />
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  Modal,
  useModal,
  VButton,
  VormInput,
  VormInputOption,
  Icon,
} from "@si/vue-lib/design-system";
import { onBeforeUnmount, onMounted, ref } from "vue";
import { BifrostComponentInList } from "@/workers/types/entity_kind_types";
import { getAssetIcon, getAssetColor } from "./util";

const components = ref<BifrostComponentInList[]>([]);

const borderStyle = (component: BifrostComponentInList) => {
  const color = getAssetColor(component.schemaCategory);
  return `border-color: ${color}`;
};

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close: closeModal } = useModal(modalRef);

function open(selectedComponents: BifrostComponentInList[]) {
  components.value = selectedComponents;
  openModal();
}
function close() {
  components.value = [];
  closeModal();
}

const mode = ref(DeleteMode.Delete);

async function onConfirm() {
  close();
  emit("delete", mode.value);
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});
onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

const onKeyDown = async (e: KeyboardEvent) => {
  if (e.key === "Enter" && modalRef.value?.isOpen) {
    onConfirm();
  }
};

const emit = defineEmits<{
  (e: "delete", value: DeleteMode): void;
}>();

defineExpose({ open, close });
</script>

<script lang="ts">
export enum DeleteMode {
  Delete,
  Remove,
}
</script>
