<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center text-sm relative p-2xs pl-xs min-w-0 w-full border border-transparent cursor-pointer',
        selected
          ? 'dark:bg-action-900 bg-action-100 border-action-500 dark:border-action-300'
          : 'dark:border-neutral-700',
        outlined
          ? ' border-action-500 dark:border-action-300'
          : 'dark:border-neutral-700',
      )
    "
  >
    <div class="flex-grow min-w-0" @click="() => viewStore.selectView(view.id)">
      <div class="flex flex-col">
        <TruncateWithTooltip class="w-full">
          <span class="text-sm">
            {{ view.name }}
          </span>
        </TruncateWithTooltip>
        <div class="flex flex-row justify-end gap-[.25em]">
          <div
            class="flex place-content-center items-center min-w-fit text-xs gap-0.5 truncate text-success-600 font-bold"
          >
            <div>{{ viewStore.viewStats[view.id]?.components }}</div>
            <Icon name="check-hex-outline" tone="success" size="sm" />
          </div>
          <div
            v-if="viewStore.viewStats[view.id]?.components || 0 > 0"
            class="flex place-content-center items-center min-w-fit x text-xs gap-0.5 truncate text-success-600 font-bold"
          >
            <div>{{ viewStore.viewStats[view.id]?.resources }}</div>
            <Icon name="check-hex" tone="success" size="sm" />
          </div>
          <div
            v-if="viewStore.viewStats[view.id]?.failed || 0 > 0"
            class="flex place-content-center items-center min-w-fit text-xs gap-0.5 truncate text-destructive-600 font-bold"
          >
            <div>{{ viewStore.viewStats[view.id]?.failed }}</div>
            <Icon name="x-hex-outline" tone="destructive" size="sm" />
          </div>
        </div>
      </div>
    </div>
    <DropdownMenu ref="contextMenuRef" :forceAbove="false" forceAlignRight>
      <DropdownMenuItem
        :onSelect="
          () => {
            modalRef?.open();
          }
        "
        label="Rename"
      />
      <DropdownMenuItem
        :disabled="
          viewStore.selectedViewId === view.id && !viewStore.viewNodes[view.id]
        "
        label="Add to Diagram"
        @click="(e: MouseEvent) => {
          if (viewStore.selectedViewId === view.id && !viewStore.viewNodes[view.id]) return;
          onAdd(view.id, e);
        }"
      />
      <DropdownMenuItem
        label="Open View in Diagram"
        :onSelect="
          () => {
            viewStore.selectView(view.id);
          }
        "
      />
      <DropdownMenuItem
        label="Inspect View in Outliner"
        :onSelect="
          () => {
            viewStore.setOutlinerView(view.id);
          }
        "
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

    <template v-if="addingView">
      <Teleport to="body">
        <div
          ref="mouseNode"
          class="fixed top-0 pointer-events-none translate-x-[-50%] translate-y-[-50%] z-100"
        >
          <NodeSkeleton color="#9d00ff" />
        </div>
      </Teleport>
    </template>
  </div>
</template>

<script lang="ts" setup>
import {
  ref,
  watch,
  computed,
  onMounted,
  onBeforeUnmount,
  nextTick,
} from "vue";

import {
  Modal,
  VormInput,
  DropdownMenu,
  DropdownMenuItem,
  TruncateWithTooltip,
  Icon,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { windowListenerManager } from "@si/vue-lib";
import { ViewDescription } from "@/api/sdf/dal/views";
import { useViewsStore } from "@/store/views.store";
import NodeSkeleton from "@/components/NodeSkeleton.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const viewStore = useViewsStore();

const props = defineProps<{
  selected?: boolean;
  outlined?: boolean;
  view: ViewDescription;
}>();

const mouseNode = ref();

const updateMouseNode = (e: MouseEvent) => {
  if (mouseNode.value) {
    const mouseX = e.clientX;
    const mouseY = e.clientY;
    mouseNode.value.style.left = `${mouseX}px`;
    mouseNode.value.style.top = `${mouseY}px`;
  }
};

const onMouseDown = (e: MouseEvent) => {
  updateMouseNode(e);
  if (viewStore.addComponentId) {
    viewStore.cancelAdd();
  }
};

const onMouseMove = (e: MouseEvent) => {
  updateMouseNode(e);
};

onMounted(() => {
  windowListenerManager.addEventListener("mousemove", onMouseMove);
  windowListenerManager.addEventListener("mousedown", onMouseDown);
});

onBeforeUnmount(() => {
  windowListenerManager.removeEventListener("mousemove", onMouseMove);
  windowListenerManager.removeEventListener("mousedown", onMouseDown);
});

const addingView = computed(() => {
  if (viewStore.addViewId)
    return viewStore.viewList.find((v) => v.id === viewStore.addViewId);
  return undefined;
});

function onAdd(id: string, e: MouseEvent) {
  // cannot dupe views
  if (Object.keys(viewStore.viewNodes).includes(id)) return;

  if (viewStore.addViewId === id) {
    viewStore.cancelAdd();
  } else {
    viewStore.addViewId = id;
    if (e) {
      nextTick(() => {
        updateMouseNode(e);
      });
    }
  }
}

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
</script>
