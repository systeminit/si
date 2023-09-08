<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ResizablePanel
    rememberSizeKey="changeset-and-asset"
    side="left"
    :minSize="250"
  >
    <template #subpanel1>
      <ComponentOutline
        class=""
        :fixesAreRunning="fixesAreRunning"
        @right-click-item="onOutlineRightClick"
      />
    </template>
    <template #subpanel2>
      <AssetPalette
        class="border-t dark:border-neutral-600"
        :fixesAreRunning="fixesAreRunning"
      />
    </template>
  </ResizablePanel>

  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <!--div
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
    </div-->
    <FixProgressOverlay />
    <GenericDiagram
      v-if="diagramNodes"
      ref="diagramRef"
      :customConfig="diagramCustomConfig"
      :nodes="diagramNodes"
      :edges="diagramEdges"
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

  <ResizablePanel
    rememberSizeKey="details-panel"
    side="right"
    :defaultSize="380"
    :minSize="300"
    :disableSubpanelResizing="!changesPanelRef?.isOpen"
  >
    <template #subpanel1>
      <Collapsible
        ref="changesPanelRef"
        contentClasses="grow relative"
        :defaultOpen="openCollapsible"
        class="flex flex-col h-full"
      >
        <template #label>
          <span
            class="flex flex-row items-center justify-center w-full text-neutral-400 gap-2"
          >
            <strong class="grow uppercase text-lg my-2">
              <template v-if="isHead">Applied Changes</template>
              <template v-else>Changes</template>
            </strong>
            <template v-if="!isHead">
              <ApplyChangeSetButton />
              <strong
                class="text-action-300 bg-action-100 text-lg rounded-2xl px-3 border border-action-300"
              >
                {{
                  1 +
                  diffs.length +
                  (changeSetStore.selectedChangeSet?.actions?.length ?? 0)
                }}
              </strong>
            </template>
          </span>
        </template>

        <ApplyHistory v-if="isHead" />
        <TabGroup
          v-else
          rememberSelectedTabKey="proposed_right"
          trackingSlug="actions_applied"
        >
          <TabGroupItem label="Proposed" slug="actions_proposed">
            <Collapsible
              as="div"
              contentAs="ul"
              :defaultOpen="false"
              hideBottomBorderWhenOpen
            >
              <template #label>
                <div class="flex flex-col min-w-0 grow">
                  <span class="font-bold truncate flex flex-row">
                    <span>Change Set Created</span>
                  </span>

                  <span class="truncate flex flex-row text-neutral-400">
                    {{
                      isHead ? "head" : changeSetStore.selectedChangeSet?.name
                    }}
                  </span>
                </div>
              </template>

              <template #default>
                <div class="px-5 text-neutral-400">
                  {{ isHead ? "head" : changeSetStore.selectedChangeSet?.name }}
                </div>
              </template>
            </Collapsible>

            <Collapsible
              v-for="diff in diffs"
              :key="diff.componentId"
              as="div"
              :defaultOpen="false"
              hideBottomBorderWhenOpen
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
            </Collapsible>

            <div
              v-for="action in changeSetStore.selectedChangeSet?.actions ?? []"
              :key="action.id"
              :class="
                clsx(
                  'border-b',
                  themeClasses('border-neutral-200', 'border-neutral-600'),
                )
              "
            >
              <ActionSprite :action="action" @remove="removeAction(action)" />
            </div>
            <div
              v-if="changeSetStore.selectedChangeSet?.actions?.length"
              class="p-4 italic !delay-0 !duration-0 hidden first:block"
            >
              <div class="pb-sm">No actions were chosen at this time.</div>
            </div>
          </TabGroupItem>

          <TabGroupItem label="Applied" slug="actions_applied">
            <ApplyHistory />
          </TabGroupItem>
        </TabGroup>
      </Collapsible>
    </template>

    <template #subpanel2>
      <div class="flex flex-col h-full">
        <SidebarSubpanelTitle>Selected Asset(s)</SidebarSubpanelTitle>

        <div class="flex-1">
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
              @delete="triggerDeleteSelection"
              @restore="triggerRestoreSelection"
            />
          </template>
          <template v-else-if="selectedComponentIds.length">
            <MultiSelectDetailsPanel />
          </template>
          <template v-else>
            <div class="flex flex-col items-center text-neutral-400">
              <EmptyStateIcon name="no-assets" class="mt-3" />
              <span class="text-xl dark:text-neutral-300"
                >No Assets Selected</span
              >
              <div class="capsize px-xs py-md italic text-sm text-center">
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
    </template>
  </ResizablePanel>

  <Modal ref="actionBlockedModalRef" :title="actionBlockedModalTitle">
    <Stack spacing="sm">
      <p>
        {{ actionBlockedModalText }}
      </p>

      <div class="flex space-x-sm justify-end">
        <VButton tone="action" @click="closeDeleteBlockedModal"> Ok</VButton>
      </div>
    </Stack>
  </Modal>

  <Modal ref="confirmDeleteModalRef" title="Are you sure?">
    <Stack spacing="sm">
      <template v-if="selectedEdge">
        <p>You're about to delete the following edge:</p>
        <EdgeCard :edgeId="selectedEdge.id" />
      </template>
      <template v-else>
        <p>You're about to delete the following component(s):</p>
        <Stack spacing="xs">
          <ComponentCard
            v-for="component in deletableSelectedComponents"
            :key="component.id"
            :componentId="component.id"
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
import { computed, onMounted, ref, watch } from "vue";
import plur from "plur";
import {
  Collapsible,
  VButton,
  Modal,
  Stack,
  TabGroup,
  TabGroupItem,
  DropdownMenu,
  DropdownMenuItemObjectDef,
  ResizablePanel,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import ComponentDetails from "@/components/ComponentDetails.vue";
import {
  ComponentId,
  EdgeId,
  useComponentsStore,
} from "@/store/components.store";
import { useFixesStore } from "@/store/fixes.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { Action } from "@/api/sdf/dal/change_set";
import ActionSprite from "@/components/ActionSprite.vue";
import SidebarSubpanelTitle from "@/components/SidebarSubpanelTitle.vue";
import { nilId } from "@/utils/nilId";
import FixProgressOverlay from "@/components/FixProgressOverlay.vue";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";
import ApplyHistory from "../ApplyHistory.vue";
import AssetPalette from "../AssetPalette.vue";
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
import ComponentOutline from "../ComponentOutline/ComponentOutline.vue";
import EdgeDetailsPanel from "../EdgeDetailsPanel.vue";
import MultiSelectDetailsPanel from "../MultiSelectDetailsPanel.vue";
import ComponentCard from "../ComponentCard.vue";
import EdgeCard from "../EdgeCard.vue";
import EmptyStateIcon from "../EmptyStateIcon.vue";

const changeSetStore = useChangeSetsStore();
const fixesStore = useFixesStore();

const fixesAreRunning = computed(
  () =>
    fixesStore.fixesAreInProgress ||
    changeSetStore.getRequestStatus("APPLY_CHANGE_SET").value.isPending,
);

const diffs = computed(() => {
  const arr = Object.values(componentsStore.componentsById)
    .filter((c) => c.changeStatus !== "unmodified")
    .map((c) => {
      let updatedAt = c.updatedInfo.timestamp;
      if (c.changeStatus === "added") {
        updatedAt = c.createdInfo.timestamp;
      } else if (c.changeStatus === "deleted" && c.deletedInfo) {
        updatedAt = c.deletedInfo.timestamp;
      }

      return {
        componentId: c.id,
        status: c.changeStatus,
        updatedAt,
      };
    });
  arr.sort(
    (a, b) => new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
  );
  return arr;
});

const isHead = computed(() => changeSetStore.selectedChangeSetId === nilId());

const openCollapsible = ref(true);

onMounted(() => {
  if (isHead.value) {
    openCollapsible.value = !!window.localStorage.getItem("applied-changes");
    window.localStorage.removeItem("applied-changes");
  } else {
    openCollapsible.value = false;
  }
});

const diagramRef = ref<InstanceType<typeof GenericDiagram>>();
const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const removeAction = (action: Action) => {
  changeSetStore.REMOVE_ACTION(action.id);
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
    await componentsStore.DELETE_COMPONENTS(selectedComponentIds.value);
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
  const disabled = fixesStore.fixesAreInProgress;
  if (selectedEdgeId.value) {
    // single selected edge
    if (selectedEdge.value?.changeStatus === "deleted") {
      items.push({
        label: "Restore edge",
        icon: "trash-restore",
        onSelect: triggerRestoreSelection,
        disabled,
      });
    } else {
      items.push({
        label: "Delete edge",
        icon: "trash",
        onSelect: triggerDeleteSelection,
        disabled,
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
        disabled,
      });
    } else {
      items.push({
        label: `Delete ${typeDisplayName()} "${
          selectedComponent.value.displayName
        }"`,
        icon: "trash",
        onSelect: triggerDeleteSelection,
        disabled,
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
        disabled,
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
        disabled,
      });
    }
  }

  if (selectedComponent.value?.resource.data) {
    items.push({
      label: "Refresh resource",
      icon: "refresh",
      onSelect: refreshResourceForSelectedComponent,
      disabled,
    });
  }
  return items;
});

const refreshResourceForSelectedComponent = () => {
  if (selectedComponent.value?.id) {
    componentsStore.REFRESH_RESOURCE_INFO(selectedComponent.value.id);
  }
};

const changesPanelRef = ref<InstanceType<typeof Collapsible>>();
</script>
