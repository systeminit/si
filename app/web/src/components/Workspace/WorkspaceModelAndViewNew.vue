<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="changeset-and-asset" side="left" :min-size="250">
    <div class="flex flex-col h-full">
      <div
        :style="{ height: `${topLeftPanel.height}px` }"
        class="relative flex-shrink-0"
      >
        <ComponentOutline class="" @right-click-item="onOutlineRightClick" />
      </div>

      <SiPanelResizer
        panel-side="bottom"
        :style="{ top: `${topLeftPanel.height}px` }"
        class="w-full"
        @resize-start="topLeftPanel.onResizeStart"
        @resize-move="topLeftPanel.onResizeMove"
        @resize-reset="topLeftPanel.resetSize"
      />

      <div v-if="!isViewMode" class="relative flex-grow">
        <AssetPalette class="border-t dark:border-neutral-600" />
      </div>
    </div>
  </SiPanel>

  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <div
      v-if="!statusStore.globalStatus.isUpdating && isViewMode"
      :class="
        clsx(
          'absolute z-20 left-0 right-0 mx-4 mt-3 p-xs',
          'bg-white dark:bg-neutral-800 dark:text-white border border-neutral-300 dark:border-neutral-600',
          'shadow-md rounded-md font-bold text-center',
        )
      "
    >
      <ReadOnlyBanner show-refresh-all-button />
    </div>
    <GlobalStatusOverlay v-else />
    <GenericDiagram
      v-if="diagramNodes"
      ref="diagramRef"
      :custom-config="diagramCustomConfig"
      :nodes="diagramNodes"
      :edges="diagramEdges"
      :read-only="isViewMode"
      @insert-element="onDiagramInsertElement"
      @hover-element="onDiagramHoverElement"
      @move-element="onDiagramMoveElement"
      @resize-element="onDiagramResizeElement"
      @group-elements="onGroupElements"
      @draw-edge="onDrawEdge"
      @delete-elements="onDiagramDelete"
      @update:selection="onDiagramUpdateSelection"
      @right-click-element="onRightClickElement"
    />

    <DropdownMenu ref="contextMenuRef" :items="rightClickMenuItems" />
  </div>

  <SiPanel
    remember-size-key="details-panel"
    side="right"
    :default-size="380"
    :min-size="300"
  >
    <div class="flex flex-col h-full">
      <span
        class="flex flex-row items-center w-full p-3 text-neutral-400 border-b dark:border-neutral-500"
      >
        <strong class="grow uppercase text-md">Changes</strong>
        <strong
          class="text-action-300 mx-2 bg-action-100 text-lg rounded-2xl px-3 border border-action-300"
          >{{ 1 + diffs.length + fixesStore.recommendations.length }}</strong
        >
      </span>
      <div :style="{ height: `${topRightPanel.height}px` }" class="relative">
        <TabGroup
          ref="proposedRightTabGroupRef"
          remember-selected-tab-key="proposed_right"
          tracking-slug="recommendations_applied"
        >
          <TabGroupItem
            v-if="!isHead"
            label="Proposed"
            slug="recommendations_proposed"
          >
            <ApplyChangeSetButton
              :recommendations="recommendationsToExecute"
              @applied-change-set="appliedRecommendations"
            />
            <SiCollapsible
              as="div"
              content-as="ul"
              :default-open="false"
              hide-bottom-border-when-open
            >
              <template #label>
                <div class="flex flex-col min-w-0 grow">
                  <span class="font-bold truncate flex flex-row">
                    <span>Change Set Created</span>
                  </span>

                  <span class="truncate flex flex-row text-neutral-400">
                    {{ changeSetStore.selectedChangeSet?.name }}
                  </span>
                </div>
              </template>

              <template #default>
                <div class="px-5 text-neutral-400">
                  {{ changeSetStore.selectedChangeSet?.name }}
                </div>
              </template>
            </SiCollapsible>

            <SiCollapsible
              v-for="diff in diffs"
              :key="diff.componentId"
              as="div"
              content-as="ul"
              :default-open="false"
              hide-bottom-border-when-open
            >
              <template #label>
                <div class="flex flex-col min-w-0 grow">
                  <span class="font-bold truncate flex flex-row">
                    <span v-if="diff.status === 'added'">Added</span>
                    <span v-if="diff.status === 'deleted'">Removed</span>
                    <span v-if="diff.status === 'modified'">Modified</span>
                    <span
                      >&nbsp;{{
                        componentsStore.componentsById[diff.componentId]
                          ?.schemaName
                      }}
                      Asset
                      {{
                        componentsStore.componentsById[diff.componentId]
                          ?.displayName
                      }}</span
                    >
                  </span>

                  <span class="truncate flex flex-row text-neutral-400">
                    {{
                      componentsStore.componentsById[diff.componentId]
                        ?.displayName
                    }}
                  </span>
                </div>
              </template>

              <template #default>
                <div class="px-5 text-neutral-400">
                  {{
                    componentsStore.componentsById[diff.componentId]
                      ?.displayName
                  }}
                </div>
              </template>
            </SiCollapsible>

            <li
              v-for="recommendation in fixesStore.recommendations"
              :key="`${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`"
            >
              <RecommendationSprite
                :key="`${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`"
                :recommendation="recommendation"
                :selected="
                  recommendationSelection[
                    `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`
                  ]
                "
                @click.stop
                @toggle="toggleRecommendation($event, recommendation)"
              />
            </li>
            <li
              v-if="fixesStore.recommendations.length === 0"
              class="p-4 italic !delay-0 !duration-0 hidden first:block"
            >
              <div class="pb-sm">
                No recommendations are available at this time.
              </div>
            </li>
          </TabGroupItem>

          <TabGroupItem label="Applied" slug="recommendations_applied">
            <ApplyHistory />
          </TabGroupItem>
        </TabGroup>
        <SiPanelResizer
          panel-side="bottom"
          style="width: 100%; bottom: 0"
          @resize-start="topRightPanel.onResizeStart"
          @resize-move="topRightPanel.onResizeMove"
          @resize-reset="topRightPanel.resetSize"
        />
      </div>

      <!-- {{ selectedComponentId }} {{ selectedEdgeId }} -->
      <div class="half">
        <SidebarSubpanelTitle>Selected Asset(s)</SidebarSubpanelTitle>

        <template v-if="selectedEdge">
          <EdgeDetailsPanel
            @delete="triggerDeleteSelection"
            @restore="triggerRestoreSelection"
          />
        </template>
        <template v-else-if="selectedComponent">
          <ComponentDetails
            v-if="selectedComponent"
            :key="selectedComponent.id"
            :is-view-mode="isViewMode"
            @delete="triggerDeleteSelection"
            @restore="triggerRestoreSelection"
          />
        </template>
        <template v-else-if="selectedComponentIds.length">
          <MultiSelectDetailsPanel />
        </template>
        <template v-else>
          <div class="flex flex-col items-center text-neutral-400">
            <NoAssets class="mt-3" />
            <span class="text-xl">No Assets Selected</span>
            <div class="capsize px-xs py-md mt-xs italic text-sm text-center">
              <template v-if="componentsStore.allComponents.length === 0">
                Your model is currently empty.
              </template>
              <template v-else
                >Click something on the diagram to select it.
              </template>
            </div>
          </div>
        </template>
      </div>
    </div>
  </SiPanel>

  <Modal ref="actionBlockedModalRef" :title="actionBlockedModalTitle">
    <Stack space="sm">
      <p>
        {{ actionBlockedModalText }}
      </p>

      <div class="flex space-x-sm justify-end">
        <VButton tone="action" @click="closeDeleteBlockedModal"> Ok</VButton>
      </div>
    </Stack>
  </Modal>

  <Modal ref="confirmDeleteModalRef" title="Are you sure?">
    <Stack space="sm">
      <template v-if="selectedEdge">
        <p>You're about to delete the following edge:</p>
        <EdgeCard :edge-id="selectedEdge.id" />
      </template>
      <template v-else>
        <p>You're about to delete the following component(s):</p>
        <Stack spacing="xs">
          <ComponentCard
            v-for="component in deletableSelectedComponents"
            :key="component.id"
            :component-id="component.id"
          />
        </Stack>
      </template>

      <p>
        Items that exist on HEAD will be marked for deletion, and removed from
        the model when this change set is merged. Items that were created in
        this changeset will be deleted immediately.
      </p>

      <div class="flex space-x-sm justify-end">
        <VButton
          icon="x"
          tone="shade"
          variant="ghost"
          @click="confirmDeleteModalRef?.close()"
        >
          Cancel
        </VButton>
        <VButton icon="trash" tone="destructive" @click="onConfirmDelete">
          Confirm
        </VButton>
      </div>
    </Stack>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref, watch, reactive } from "vue";
import { useRoute } from "vue-router";
import plur from "plur";
import clsx from "clsx";
import {
  VButton,
  Modal,
  Stack,
  TabGroup,
  TabGroupItem,
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import ComponentDetails from "@/components/ComponentDetails.vue";
import SiCollapsible from "@/components/SiCollapsible.vue";
import {
  ComponentId,
  EdgeId,
  useComponentsStore,
} from "@/store/components.store";
import NoAssets from "@/assets/images/no-assets.svg?component";

import SiPanel from "@/components/SiPanel.vue";
import { useStatusStore } from "@/store/status.store";
import { Recommendation, useFixesStore } from "@/store/fixes.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import RecommendationSprite from "@/components/RecommendationSprite2.vue";
import SiPanelResizer from "@/components/SiPanelResizer.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import { nilId } from "@/utils/nilId";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";
import ApplyHistory from "../ApplyHistory.vue";
import AssetPalette from "../AssetPalette2.vue";
import {
  InsertElementEvent,
  MoveElementEvent,
  DrawEdgeEvent,
  DeleteElementsEvent,
  RightClickElementEvent,
  DiagramNodeData,
  DiagramGroupData,
  GroupEvent,
  SelectElementEvent,
  ResizeElementEvent,
  DiagramEdgeData,
  HoverElementEvent,
} from "../GenericDiagram/diagram_types";
import ComponentOutline from "../ComponentOutline/ComponentOutline2.vue";
import GlobalStatusOverlay from "../GlobalStatusOverlay.vue";
import EdgeDetailsPanel from "../EdgeDetailsPanel.vue";
import MultiSelectDetailsPanel from "../MultiSelectDetailsPanel.vue";
import ComponentCard from "../ComponentCard.vue";
import EdgeCard from "../EdgeCard.vue";
import ReadOnlyBanner from "../ReadOnlyBanner.vue";

const statusStore = useStatusStore();
const changeSetStore = useChangeSetsStore();
const fixesStore = useFixesStore();

const diffs = computed(() => {
  const arr = Object.values(componentsStore.componentsById)
    .filter((c) => c.changeStatus !== "unmodified")
    .map((c) => ({
      componentId: c.id,
      status: c.changeStatus,
      updatedAt: c.updatedInfo.timestamp,
    }));
  arr.sort(
    (a, b) => new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
  );
  return arr;
});

const proposedRightTabGroupRef = ref<InstanceType<typeof TabGroup>>();
const appliedRecommendations = () => {
  proposedRightTabGroupRef.value?.selectTab("recommendations_applied");
};

const recommendationSelection = ref<Record<string, boolean>>({});
const recommendationsToExecute = computed(() => {
  return fixesStore.recommendations.filter((recommendation) => {
    const key = `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`;
    return recommendationSelection.value[key] ? recommendation : null;
  });
});
const { recommendations } = storeToRefs(fixesStore);
watch(recommendations, (r) => {
  const keys = new Set(...Object.keys(recommendationSelection.value));
  for (const recommendation of r) {
    const key = `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`;
    keys.delete(key);
    recommendationSelection.value[key] =
      recommendationSelection.value[key] ?? true;
  }

  for (const key of keys) {
    delete recommendationSelection.value[key];
  }
});

// TODO: we'll very likely split view mode from compose mode again, so this is just temporary
// but for now we watch if the route is for view mode, and if so, switch to head and toggle a few things
const isHead = computed(() =>
  [null, nilId()].includes(changeSetStore.selectedChangeSetId),
);
const isViewMode = computed(
  (_) =>
    isHead.value ||
    changeSetStore.getRequestStatus("APPLY_CHANGE_SET2").value.isPending,
);

const diagramRef = ref<InstanceType<typeof GenericDiagram>>();
const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const toggleRecommendation = (c: boolean, recommendation: Recommendation) => {
  const key = `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`;
  recommendationSelection.value[key] = c;
};

const componentsStore = useComponentsStore();
// TODO: probably want to get more generic component data and then transform into diagram nodes
const diagramEdges = computed(() => {
  // Note(victor): The code below checks whether was only created implicitly, through inheritance from an aggregation frame
  // In the future, it would make more sense for this to be stored on the database
  const validEdges = _.filter(componentsStore.diagramEdges, (edge) => {
    return (
      componentsStore.componentsByNodeId[edge.toNodeId] !== undefined &&
      componentsStore.componentsByNodeId[edge.fromNodeId] !== undefined
    );
  });
  const edges = _.map(validEdges, (edge) => {
    edge.isInvisible = false;

    const toNodeParentId =
      componentsStore.componentsByNodeId[edge.toNodeId]?.parentNodeId;

    if (toNodeParentId) {
      const toNodeParentComp =
        componentsStore.componentsByNodeId[toNodeParentId];

      if (toNodeParentComp?.nodeType === "aggregationFrame") {
        if (edge.fromNodeId === toNodeParentComp.nodeId) {
          edge.isInvisible ||= true;
        }
      }
    }

    const fromNodeParentId =
      componentsStore.componentsByNodeId[edge.fromNodeId]?.parentNodeId;

    if (fromNodeParentId) {
      const fromParentComp =
        componentsStore.componentsByNodeId[fromNodeParentId];
      if (fromParentComp?.nodeType === "aggregationFrame") {
        if (edge.toNodeId === fromParentComp.nodeId) {
          edge.isInvisible ||= true;
        }
      }
    }

    return edge;
  });

  return edges;
});
const diagramNodes = computed(() => componentsStore.diagramNodes);

const selectedComponentId = computed(() => componentsStore.selectedComponentId);
const selectedComponentIds = computed(
  () => componentsStore.selectedComponentIds,
);
const selectedComponents = computed(() => componentsStore.selectedComponents);

const selectedEdgeId = computed(() => componentsStore.selectedEdgeId);
const selectedEdge = computed(() => componentsStore.selectedEdge);

const diagramCustomConfig = {};

const selectedComponent = computed(() => componentsStore.selectedComponent);

const insertCallbacks: Record<string, () => void> = {};

watch(
  () => componentsStore.selectedInsertSchemaId,
  () => {
    if (componentsStore.selectedInsertSchemaId) {
      diagramRef.value?.beginInsertElement("node");
    } else {
      diagramRef.value?.endInsertElement();
    }
  },
);

watch([diagramNodes, diagramEdges], () => {
  // TODO: this should be firing off the callback only when we find the matching new node, but we dont have the new ID yet
  _.each(insertCallbacks, (insertCallback, newNodeId) => {
    insertCallback();
    delete insertCallbacks[newNodeId];
  });
});

async function onDrawEdge(newEdge: DrawEdgeEvent) {
  const fromNodeId = newEdge.fromSocket.parent.def.id;
  const fromSocketId = newEdge.fromSocket.def.id;
  const toNodeId = newEdge.toSocket.parent.def.id;
  const toSocketId = newEdge.toSocket.def.id;

  const equivalentEdge = diagramEdges.value.find(
    (e) =>
      e.fromNodeId === fromNodeId &&
      e.fromSocketId === fromSocketId &&
      e.toNodeId === toNodeId &&
      e.toSocketId === toSocketId,
  );

  if (equivalentEdge) {
    await componentsStore.RESTORE_EDGE(equivalentEdge.id);
  } else {
    await componentsStore.CREATE_COMPONENT_CONNECTION(
      {
        nodeId: fromNodeId,
        socketId: fromSocketId,
      },
      {
        nodeId: toNodeId,
        socketId: toSocketId,
      },
    );
  }
}

async function onDiagramInsertElement(e: InsertElementEvent) {
  if (!componentsStore.selectedInsertSchemaId)
    throw new Error("missing insert selection metadata");

  const schemaId = componentsStore.selectedInsertSchemaId;
  componentsStore.selectedInsertSchemaId = null;

  let parentId;

  if (e.parent) {
    const parentComponent = Object.values(componentsStore.componentsById).find(
      (c) => c.nodeId === e.parent,
    );
    if (
      parentComponent &&
      (parentComponent.nodeType !== "aggregationFrame" ||
        schemaId === parentComponent.schemaId)
    ) {
      parentId = e.parent;
    }
  }

  // TODO These ids should be number from the start.
  const createReq = await componentsStore.CREATE_COMPONENT(
    schemaId,
    e.position,
    parentId,
  );

  // TODO(nick,theo): consider what to do upon failure.
  if (createReq.result.success) {
    insertCallbacks[createReq.result.data.componentId] = e.onComplete;
  }
}

function onDiagramResizeElement(e: ResizeElementEvent) {
  if (!e.isFinal) return;
  if (e.element instanceof DiagramGroupData) {
    componentsStore.SET_COMPONENT_DIAGRAM_POSITION(
      e.element.def.id,
      e.position,
      e.size,
    );
  }
}

function onDiagramMoveElement(e: MoveElementEvent) {
  // this gets called many times during a move, with e.isFinal telling you if the drag is in progress or complete
  // eventually we will want to send those to the backend for realtime multiplayer
  // But for now we just send off the final position
  if (!e.isFinal) return;
  if (
    e.element instanceof DiagramNodeData ||
    e.element instanceof DiagramGroupData
  ) {
    componentsStore.SET_COMPONENT_DIAGRAM_POSITION(
      e.element.def.id,
      e.position,
    );
  }
}

function onDiagramUpdateSelection(newSelection: SelectElementEvent) {
  if (
    newSelection.elements.length === 1 &&
    newSelection.elements[0] instanceof DiagramEdgeData
  ) {
    componentsStore.setSelectedEdgeId(newSelection.elements[0].def.id);
  } else {
    const validComponentIds = _.compact(
      newSelection.elements.map((el) => {
        if (el instanceof DiagramNodeData || el instanceof DiagramGroupData) {
          return el.def.componentId;
        }
        return undefined;
      }),
    );
    componentsStore.setSelectedComponentId(validComponentIds);
  }
}

const confirmDeleteModalRef = ref<InstanceType<typeof Modal>>();
const actionBlockedModalRef = ref<InstanceType<typeof Modal>>();
const actionBlockedModalTitle = ref<string>();
const actionBlockedModalText = ref<string>();

function closeDeleteBlockedModal() {
  actionBlockedModalRef.value?.close();
}

const deletableSelectedComponents = computed(() => {
  return _.reject(
    componentsStore.selectedComponents,
    (c) => c.changeStatus === "deleted",
  );
});
const restorableSelectedComponents = computed(() => {
  return _.filter(
    componentsStore.selectedComponents,
    (c) => c.changeStatus === "deleted",
  );
});

function onDiagramDelete(_e: DeleteElementsEvent) {
  // delete event includes what to delete, but it's the same as current selection
  triggerDeleteSelection();
}

function triggerDeleteSelection() {
  // event is triggered regardless of selection
  // in some cases we may want to ignore it
  if (selectedEdge.value) {
    if (selectedEdge.value?.changeStatus === "deleted") return;
  } else {
    // TODO: more logic to decide if modal is necessary for other situations
    if (!deletableSelectedComponents.value.length) return;
  }

  const deletionSubjectHasChildren =
    selectedComponents.value?.filter((el) => {
      const activeChildren = el.childNodeIds.filter((childId) => {
        const child = componentsStore.componentsByNodeId[childId];
        return _.isNil(child?.deletedInfo);
      });
      return activeChildren.length > 0;
    }).length > 0;

  if (deletionSubjectHasChildren) {
    actionBlockedModalTitle.value = "Can't delete component";
    actionBlockedModalText.value =
      "You cannot delete a frame that still has children. Delete them before proceeding.";
    actionBlockedModalRef.value?.open();
    return;
  }

  confirmDeleteModalRef.value?.open();
}

function onConfirmDelete() {
  // TODO: show loading in modal, and close after complete
  executeDeleteSelection();
  confirmDeleteModalRef.value?.close();
}

async function executeDeleteSelection() {
  if (selectedEdgeId.value) {
    await componentsStore.DELETE_EDGE(selectedEdgeId.value);
  } else if (selectedComponentIds.value) {
    for (const componentId of selectedComponentIds.value) {
      await componentsStore.DELETE_COMPONENT(componentId);
    }
  }
  componentsStore.setSelectedComponentId(null);
}

async function triggerRestoreSelection() {
  if (selectedEdgeId.value) {
    await componentsStore.RESTORE_EDGE(selectedEdgeId.value);
  } else if (selectedComponentIds.value) {
    // Block restoring child of deleted frame
    const parentIds = _.compact(
      _.map(
        selectedComponentIds.value,
        (id) => componentsStore.componentsById[id]?.parentId,
      ),
    );

    const hasDeletedParent = parentIds.find(
      (id) => !_.isNil(componentsStore.componentsById[id]?.deletedInfo),
    );

    if (hasDeletedParent) {
      actionBlockedModalTitle.value = "Can't restore component";
      actionBlockedModalText.value =
        "You cannot restore a component inside a deleted frame. Restore the parent before restoring its children.";

      actionBlockedModalRef.value?.open();
      return;
    }

    await componentsStore.RESTORE_COMPONENTS(selectedComponentIds.value);
  }
}

function getDiagramElementKeyForComponentId(componentId?: ComponentId | null) {
  if (!componentId) return;
  const component = componentsStore.componentsById[componentId];
  if (component) {
    if (component.isGroup) {
      return DiagramGroupData.generateUniqueKey(component.nodeId);
    }
    return DiagramNodeData.generateUniqueKey(component.nodeId);
  }
}

function getDiagramElementKeyForEdgeId(edgeId?: EdgeId | null) {
  if (!edgeId) return;
  return DiagramEdgeData.generateUniqueKey(edgeId);
}

// HOVER
function onDiagramHoverElement(newHover: HoverElementEvent) {
  if (
    newHover.element instanceof DiagramNodeData ||
    newHover.element instanceof DiagramGroupData
  ) {
    componentsStore.setHoveredComponentId(newHover.element.def.componentId);
  } else if (newHover.element instanceof DiagramEdgeData) {
    componentsStore.setHoveredEdgeId(newHover.element.def.id);
  } else {
    // handles case of hovering nothing and hovering edges
    componentsStore.setHoveredComponentId(null);
  }
}

watch(
  [
    () => componentsStore.hoveredComponentId,
    () => componentsStore.hoveredEdgeId,
  ],
  () => {
    if (componentsStore.hoveredComponentId) {
      diagramRef.value?.setHoveredByKey(
        getDiagramElementKeyForComponentId(componentsStore.hoveredComponentId),
      );
    } else if (componentsStore.hoveredEdgeId) {
      diagramRef.value?.setHoveredByKey(
        getDiagramElementKeyForEdgeId(componentsStore.hoveredEdgeId),
      );
    } else {
      diagramRef.value?.setHoveredByKey(undefined);
    }
  },
);

watch(
  () => [selectedComponentIds.value, selectedEdgeId.value],
  () => {
    if (selectedComponentIds.value.length > 0) {
      const selectedComponentsKeys = _.map(
        selectedComponentIds.value,
        getDiagramElementKeyForComponentId,
      );
      diagramRef.value?.setSelectionByKey(_.compact(selectedComponentsKeys));
    } else if (selectedEdgeId.value) {
      diagramRef.value?.setSelectionByKey(
        getDiagramElementKeyForEdgeId(selectedEdgeId.value),
      );
    } else {
      diagramRef.value?.clearSelection();
    }
  },
);

function onGroupElements({ group, elements }: GroupEvent) {
  if (group.def.nodeType === "aggregationFrame") {
    const groupSchemaId =
      componentsStore.componentsByNodeId[group.def.id]?.schemaVariantId;
    elements = _.filter(elements, (e) => {
      const elementSchemaId =
        componentsStore.componentsByNodeId[e.def.id]?.schemaVariantId;

      return elementSchemaId === groupSchemaId;
    });
  }

  for (const element of elements) {
    componentsStore.CONNECT_COMPONENT_TO_FRAME(element.def.id, group.def.id);
  }
}

function onRightClickElement(rightClickEventInfo: RightClickElementEvent) {
  // TODO: make actually do something, probably also want to handle different types
  contextMenuRef.value?.open(rightClickEventInfo.e, true);
}

function onOutlineRightClick(e: MouseEvent) {
  contextMenuRef.value?.open(e, true);
}

const typeDisplayName = (action = "delete") => {
  if (selectedComponentId.value && selectedComponent.value) {
    if (selectedComponent.value.nodeType === "component") return "Component";
    else return "Frame";
  } else if (selectedComponentIds.value.length) {
    const components =
      action === "delete"
        ? deletableSelectedComponents
        : restorableSelectedComponents;

    for (const c of components.value) {
      if (c.nodeType === "component") return "Component"; // if we have both frames and components, just use the word component
    }

    return "Frame";
  } else {
    return "Component";
  }
};

const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  if (!isViewMode.value) {
    if (selectedEdgeId.value) {
      // single selected edge
      if (selectedEdge.value?.changeStatus === "deleted") {
        items.push({
          label: "Restore edge",
          icon: "trash-restore",
          onSelect: triggerRestoreSelection,
        });
      } else {
        items.push({
          label: "Delete edge",
          icon: "trash",
          onSelect: triggerDeleteSelection,
        });
      }
    } else if (selectedComponentId.value && selectedComponent.value) {
      // single selected component
      if (selectedComponent.value.changeStatus === "deleted") {
        items.push({
          label: `Restore ${typeDisplayName()} "${
            selectedComponent.value.displayName
          }"`,
          icon: "trash-restore",
          onSelect: triggerRestoreSelection,
        });
      } else {
        items.push({
          label: `Delete ${typeDisplayName()} "${
            selectedComponent.value.displayName
          }"`,
          icon: "trash",
          onSelect: triggerDeleteSelection,
        });
      }
    } else if (selectedComponentIds.value.length) {
      // Multiple selected components
      if (deletableSelectedComponents.value.length > 0) {
        items.push({
          label: `Delete ${deletableSelectedComponents.value.length} ${plur(
            typeDisplayName("delete"),
            deletableSelectedComponents.value.length,
          )}`,
          icon: "trash",
          onSelect: triggerDeleteSelection,
        });
      }
      if (restorableSelectedComponents.value.length > 0) {
        items.push({
          label: `Restore ${restorableSelectedComponents.value.length} ${plur(
            typeDisplayName("restore"),
            restorableSelectedComponents.value.length,
          )}`,
          icon: "trash-restore",
          onSelect: triggerRestoreSelection,
        });
      }
    }
  }
  if (selectedComponent.value?.resource.data) {
    items.push({
      label: "Refresh resource",
      icon: "refresh",
      onSelect: refreshResourceForSelectedComponent,
    });
  }
  return items;
});

const refreshResourceForSelectedComponent = () => {
  if (selectedComponent.value?.id) {
    componentsStore.REFRESH_RESOURCE_INFO(selectedComponent.value.id);
  }
};

const SUB_PANEL_DEFAULT_HEIGHT = 350;
const SUB_PANEL_MIN_HEIGHT = 150;

// TODO: Move panels to their own components after they stabilize a bit
const topRightPanel = reactive({
  height: SUB_PANEL_DEFAULT_HEIGHT,
  beginResizeValue: 0,

  onResizeStart() {
    topRightPanel.beginResizeValue = topRightPanel.height;
  },

  onResizeMove(delta: number) {
    const adjustedDelta = -delta;
    const newHeight = topRightPanel.beginResizeValue + adjustedDelta;

    topRightPanel.height = Math.max(newHeight, SUB_PANEL_MIN_HEIGHT);
  },

  resetSize() {
    topRightPanel.height = SUB_PANEL_DEFAULT_HEIGHT;
  },
});

const topLeftPanel = reactive({
  height: SUB_PANEL_DEFAULT_HEIGHT,
  beginResizeValue: 0,

  onResizeStart() {
    topLeftPanel.beginResizeValue = topLeftPanel.height;
  },

  onResizeMove(delta: number) {
    const adjustedDelta = -delta;
    const newHeight = topLeftPanel.beginResizeValue + adjustedDelta;

    topLeftPanel.height = Math.max(newHeight, SUB_PANEL_MIN_HEIGHT);
  },

  resetSize() {
    topLeftPanel.height = SUB_PANEL_DEFAULT_HEIGHT;
  },
});
</script>

<style lang="less" scoped>
.half {
  flex: 0%;
}
</style>
