<template>
  <component
    :is="displayAsComponentCard ? ComponentCard : 'div'"
    :class="
      clsx(
        !displayAsComponentCard && [
          'flex flex-row items-center text-sm relative p-2xs pl-xs min-w-0 w-full border border-transparent cursor-pointer',
          selected
            ? 'dark:bg-action-900 bg-action-100 border-action-500 dark:border-action-300'
            : 'dark:border-neutral-700',
          outlined
            ? ' border-action-500 dark:border-action-300'
            : 'dark:border-neutral-700',
        ],
      )
    "
    titleCard
    :component="displayAsComponentCard"
  >
    <div
      v-if="!displayAsComponentCard"
      class="flex-grow min-w-0"
      @click="() => viewsStore.selectView(view.id)"
    >
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
            <div>{{ viewsStore.viewStats[view.id]?.components }}</div>
            <Icon name="check-hex-outline" tone="success" size="sm" />
          </div>
          <div
            v-if="viewsStore.viewStats[view.id]?.components || 0 > 0"
            class="flex place-content-center items-center min-w-fit x text-xs gap-0.5 truncate text-success-600 font-bold"
          >
            <div>{{ viewsStore.viewStats[view.id]?.resources }}</div>
            <Icon name="check-hex" tone="success" size="sm" />
          </div>
          <div
            v-if="viewsStore.viewStats[view.id]?.failed || 0 > 0"
            class="flex place-content-center items-center min-w-fit text-xs gap-0.5 truncate text-destructive-600 font-bold"
          >
            <div>{{ viewsStore.viewStats[view.id]?.failed }}</div>
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
        icon="cursor"
      />
      <DropdownMenuItem
        v-if="!displayAsComponentCard"
        :disabled="
          viewsStore.selectedViewId === view.id &&
          !viewsStore.viewNodes[view.id]
        "
        label="Add to Diagram"
        icon="plus"
        @click="
          (e: MouseEvent) => {
            if (
              viewsStore.selectedViewId === view.id &&
              !viewsStore.viewNodes[view.id]
            )
              return;
            onAdd(view.id, e);
          }
        "
      />
      <DropdownMenuItem
        label="Open View in Diagram"
        icon="eye"
        :onSelect="
          () => {
            viewsStore.selectView(view.id);
          }
        "
      />
      <DropdownMenuItem
        label="Inspect View in Outliner"
        icon="bullet-list-indented"
        :onSelect="
          () => {
            viewsStore.setOutlinerView(view.id);
          }
        "
      />
      <DropdownMenuItem
        v-if="displayAsComponentCard && !selectedViaViewDetails"
        label="Remove from this View"
        icon="x-circle"
        :onSelect="() => removeFromView()"
      />
      <DropdownMenuItem
        label="Approval Requirements"
        icon="bullet-list"
        :onSelect="() => displayApprovalRequirements()"
      />
      <DropdownMenuItem
        label="Delete View"
        :disabled="viewsStore.viewListCount === 1"
        icon="trash"
        :onSelect="() => deleteView(view)"
      />
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
      saveLabel="Rename"
      title="Rename View"
      @save="updateName"
      @close="resetRename"
    >
      <VormInput
        ref="labelRef"
        v-model="viewName"
        required
        label="View Name"
        @enterPressed="updateName"
      />
    </Modal>

    <template v-if="addingView && !displayAsComponentCard">
      <Teleport to="body">
        <div
          ref="mouseNode"
          class="fixed top-0 pointer-events-none translate-x-[-50%] translate-y-[-50%] z-100"
        >
          <NodeSkeleton color="#9d00ff" />
        </div>
      </Teleport>
    </template>
  </component>
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
import { useToast } from "vue-toastification";
import { ViewDescription } from "@/api/sdf/dal/views";
import { useViewsStore } from "@/store/views.store";
import NodeSkeleton from "@/components/NodeSkeleton.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";
import ComponentCard from "./ComponentCard.vue";
import { DiagramViewData } from "./ModelingDiagram/diagram_types";

const toast = useToast();
const viewsStore = useViewsStore();

const props = defineProps<{
  selected?: boolean;
  outlined?: boolean;
  displayAsComponentCard?: DiagramViewData;
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
  if (viewsStore.addComponentId) {
    viewsStore.cancelAdd();
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
  if (viewsStore.addViewId)
    return viewsStore.viewList.find((v) => v.id === viewsStore.addViewId);
  return undefined;
});

function onAdd(id: string, e: MouseEvent) {
  // cannot dupe views
  if (Object.keys(viewsStore.viewNodes).includes(id)) return;

  if (viewsStore.addViewId === id) {
    viewsStore.cancelAdd();
  } else {
    viewsStore.addViewId = id;
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
    viewsStore.UPDATE_VIEW_NAME(props.view.id, viewName.value);
    modalRef.value?.close();
    viewName.value = "";
  }
};

const deleteView = async (view: ViewDescription) => {
  const resp = await viewsStore.DELETE_VIEW(view.id);
  if (!resp.result.success) {
    if (resp.result.statusCode === 409) {
      /* We cannot easily pass JSON data as an error, punting for now with a generic message
      const ids = resp.rawResponseError.response.data.error as ComponentId[];
      const names = ids
        .map((cId) => componentStore.allComponentsById[cId]?.def.displayName)
        .filter((name) => !!name); */
      toast(
        "Cannot delete the view. Deleting the view would cause orphan components.",
      );
    }
  }
};

const resetRename = () => {
  viewName.value = props.view.name;
};

watch(
  props.view,
  () => {
    viewName.value = props.view.name;
  },
  { immediate: true },
);

const removeFromView = () => {
  viewsStore.removeSelectedViewComponentFromCurrentView();
};

const displayApprovalRequirements = () => {
  viewsStore.setSelectedViewDetails(props.view.id);
};

const selectedViaViewDetails = computed(
  () => !!viewsStore.selectedViewDetailsId,
);
</script>
