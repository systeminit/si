<template>
  <div class="w-full h-full flex relative overflow-hidden">
    <div class="flex flex-row w-full bg-transparent">
      <SiSidebar side="left">
        <ChangeSetPanel
          v-if="!isViewMode"
          class="border-b-2 dark:border-neutral-500 mb-2"
        />

        <SiTabGroup :top-margin="0">
          <template #tabs>
            <SiTabHeader v-if="!isViewMode">Asset Palette</SiTabHeader>
            <SiTabHeader>Diagram Outline</SiTabHeader>
          </template>

          <template #panels>
            <TabPanel
              v-if="!isViewMode"
              class="flex flex-col overflow-y-hidden"
            >
              <AssetPalette @select="onSelectAssetToInsert" />
            </TabPanel>
            <TabPanel class="flex flex-col overflow-y-hidden">
              <SchematicOutline
                :selected-component-id="selectedComponentId ?? undefined"
                @select="onOutlineSelectComponent"
              />
            </TabPanel>
          </template>
        </SiTabGroup>
      </SiSidebar>

      <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
        <GenericDiagram
          v-if="diagramData"
          ref="diagramRef"
          :custom-config="diagramCustomConfig"
          :nodes="diagramData?.nodes"
          :edges="diagramData?.edges"
          :read-only="isViewMode"
          @insert-element="onDiagramInsertElement"
          @move-element="onDiagramMoveElement"
          @draw-edge="onDrawEdge"
          @delete-elements="onDiagramDelete"
          @update:selection="onDiagramUpdateSelection"
        />
      </div>

      <SiSidebar side="right">
        <ComponentDetails
          v-if="selectedComponent"
          :component-identification="selectedComponent"
          :component-name="selectedComponentLabel || 'selected component'"
        />
        <div v-else class="p-4">
          <template v-if="isViewMode">
            Select a single component to see more details
          </template>
          <template v-else>Select a single component to edit it </template>
        </div>
      </SiSidebar>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import _ from "lodash";
import SiSidebar from "@/atoms/SiSidebar.vue";
import { computed, ref, watch } from "vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import ComponentDetails from "@/organisms/ComponentDetails.vue";
import { ComponentService } from "@/service/component";
import SchematicDiagramService from "@/service/schematic-diagram";
import {
  InsertElementEvent,
  MoveElementEvent,
  DrawEdgeEvent,
  DiagramElementIdentifier,
  DeleteElementsEvent,
} from "../GenericDiagram/diagram_types";
import AssetPalette, { SelectAssetEvent } from "../AssetPalette.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";

import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";
import { useObservable } from "@vueuse/rxjs";
import { useRoute } from "vue-router";
import SchematicOutline from "../SchematicOutline.vue";
import { ChangeSetService } from "@/service/change_set";
import { SelectionService } from "@/service/selection";

import KubernetesIconRaw from "@/assets/images/3p-logos/kubernetes/kubernetes-icon.svg?raw";
import DockerIconRaw from "@/assets/images/3p-logos/docker/docker-icon.svg?raw";

const currentRoute = useRoute();

// TODO: we'll very likely split view mode from compose mode again, so this is just temporary
// but for now we watch if the route is for view mode, and if so, switch to head and toggle a few things
const isViewMode = computed(() => currentRoute.name === "workspace-view");
watch(currentRoute, () => {
  if (isViewMode.value) ChangeSetService.switchToHead();
});

const diagramRef = ref<InstanceType<typeof GenericDiagram>>();

const diagramData = SchematicDiagramService.useDiagramData();
const schemaVariants = SchematicDiagramService.useSchemaVariants();

const selectedComponentId = SelectionService.useSelectedComponentId();

const diagramCustomConfig = computed(() => ({
  icons: {
    docker: DockerIconRaw,
    kubernetes: KubernetesIconRaw,
  },
}));

const componentsListApiResponse = useObservable(
  ComponentService.listComponentsIdentification(),
);
const componentsById = computed(() => {
  if (componentsListApiResponse.value?.error) return {};
  return _.keyBy(
    componentsListApiResponse.value?.list,
    (i) => i.value.componentId,
  );
});

const selectedComponent = computed(() => {
  if (!selectedComponentId.value) return;
  return componentsById.value[selectedComponentId.value]?.value;
});
// TODO: bit weird how the label is stored split - ideally wouldnt need to pass it in anyway
const selectedComponentLabel = computed(() => {
  if (!selectedComponentId.value) return;
  return componentsById.value[selectedComponentId.value]?.label;
});

const lastInsertSelection = ref<{ schemaId: number }>();
const insertCallbacks: Record<string, () => void> = {};
function onSelectAssetToInsert(e: SelectAssetEvent) {
  // keep track of what was selected to insert
  lastInsertSelection.value = { schemaId: e.schemaId };
  diagramRef.value?.beginInsertElement("node");
}
watch(diagramData, () => {
  // TODO: this should be firing off the callback only when we find the matching new node, but we dont have the new ID yet
  _.each(insertCallbacks, (insertCallback, newNodeId) => {
    insertCallback();
    delete insertCallbacks[newNodeId];
  });
});

async function onDrawEdge(e: DrawEdgeEvent) {
  const [fromNodeId, fromSocketId] = e.fromSocketId.split("-");
  const [toNodeId, toSocketId] = e.toSocketId.split("-");

  // TODO: this is super hacky - we should not need to pass these IDs from the frontend anyway
  const sockets = _.flatMap(schemaVariants.value, (sv) => [
    ...sv.inputSockets,
    ...sv.outputSockets,
  ]);
  const socketsById = _.keyBy(sockets, (s) => s.id);

  const fromProviderId = socketsById[fromSocketId].provider.id;
  const toProviderId = socketsById[toSocketId].provider.id;

  await SchematicDiagramService.actions.createConnection({
    fromNodeId,
    fromSocketId,
    fromProviderId: fromProviderId.toString(),
    toNodeId,
    toSocketId,
    toProviderId: toProviderId.toString(),
  });
}

async function onDiagramInsertElement(e: InsertElementEvent) {
  if (!lastInsertSelection.value)
    throw new Error("missing insert selection metadata");

  await SchematicDiagramService.actions.createNode(
    lastInsertSelection.value.schemaId,
    e.position,
  );

  // TODO: we actually want the new node ID so we can watch for it in the updated data
  // but the API currently doesn't have it right away :(
  const newNodeId = +new Date();
  insertCallbacks[newNodeId] = e.onComplete;
}

function onDiagramMoveElement(e: MoveElementEvent) {
  // this gets called many times during a move, with e.isFinal telling you if the drag is in progress or complete
  // eventually we will want to send those to the backend for realtime multiplayer
  // But for now we just send off the final position
  if (!e.isFinal) return;
  SchematicDiagramService.actions.updateNodePosition(e.id, e.position);
}

function onDiagramUpdateSelection(newSelection: DiagramElementIdentifier[]) {
  // for now, we dont support multiselect anywhere outside the diagram, so we just act like nothing is selected
  if (newSelection.length !== 1) {
    SelectionService.setSelectedComponentId(null);
    return;
  }

  const selectedElement = newSelection[0];
  // we also dont support selecting things other than nodes outside the diagram
  if (selectedElement.diagramElementType !== "node") {
    SelectionService.setSelectedComponentId(null);
    return;
  }
  SelectionService.setSelectedComponentId(parseInt(selectedElement.id));
}

function onDiagramDelete(_e: DeleteElementsEvent) {
  // eslint-disable-next-line no-alert
  alert("Deletion not supported yet!");
}

function onOutlineSelectComponent(id: number) {
  SelectionService.setSelectedComponentId(id);
}

watch(
  () => selectedComponentId.value,
  () => {
    if (selectedComponentId.value) {
      diagramRef.value?.setSelection({
        diagramElementType: "node",
        id: selectedComponentId.value.toString(),
      });
    } else {
      diagramRef.value?.clearSelection();
    }
  },
);
</script>
