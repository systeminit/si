<template>
  <SiPanel remember-size-key="changeset-and-asset" side="left" :min-size="250">
    <div class="flex flex-col h-full">
      <ChangeSetPanel
        v-if="!isViewMode"
        class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
      />

      <SiTabGroup class="relative flex-grow">
        <template #tabs>
          <SiTabHeader v-if="!isViewMode">Asset Palette</SiTabHeader>
          <SiTabHeader>Diagram Outline</SiTabHeader>
        </template>

        <template #panels>
          <TabPanel v-if="!isViewMode">
            <AssetPalette @select="onSelectAssetToInsert" />
          </TabPanel>
          <TabPanel>
            <DiagramOutline
              :selected-component-id="selectedComponentId ?? undefined"
              @select="onOutlineSelectComponent"
            />
          </TabPanel>
        </template>
      </SiTabGroup>
    </div>
  </SiPanel>

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

  <SiPanel
    remember-size-key="component-details"
    side="right"
    :default-size="380"
    :min-size="300"
  >
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
  </SiPanel>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import _ from "lodash";
import { computed, ref, watch } from "vue";
import { useObservable } from "@vueuse/rxjs";
import { useRoute } from "vue-router";
import SiPanel from "@/atoms/SiPanel.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import ComponentDetails from "@/organisms/ComponentDetails.vue";
import { ComponentService } from "@/service/component";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { ChangeSetService } from "@/service/change_set";
import { SelectionService } from "@/service/selection";
import { QualificationService } from "@/service/qualification";
import DiagramService2 from "@/service/diagram2";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";
import AssetPalette, { SelectAssetEvent } from "../AssetPalette.vue";
import {
  InsertElementEvent,
  MoveElementEvent,
  DrawEdgeEvent,
  DiagramElementIdentifier,
  DeleteElementsEvent,
  DiagramStatusIcon,
} from "../GenericDiagram/diagram_types";
import DiagramOutline from "../DiagramOutline.vue";
import { LogoIcons } from "./logo_icons";

const currentRoute = useRoute();

// TODO: we'll very likely split view mode from compose mode again, so this is just temporary
// but for now we watch if the route is for view mode, and if so, switch to head and toggle a few things
const isViewMode = computed(() => currentRoute.name === "workspace-view");
watch(currentRoute, () => {
  if (isViewMode.value) ChangeSetService.switchToHead();
});

const diagramRef = ref<InstanceType<typeof GenericDiagram>>();

const rawDiagramData = DiagramService2.useDiagramData();
const qualificationSummary = QualificationService.useQualificationSummary();

const selectedComponentId = SelectionService.useSelectedComponentId();

const diagramCustomConfig = {
  icons: LogoIcons,
};

type QualificationStatus = "success" | "failure" | "running";
const qualificationStatusToIconMap: Record<
  QualificationStatus,
  DiagramStatusIcon
> = {
  success: { icon: "check", tone: "success" },
  failure: { icon: "alert", tone: "error" },
  running: { icon: "loading", tone: "info" },
};

// TODO: we'll probably want to link the qualification data to the components in the service layer / store
// so that it will be reusable elsewhere... but we'll temporarily do it here to get it working
const diagramData = computed(() => {
  return {
    ...rawDiagramData.value,
    nodes: _.map(rawDiagramData.value?.nodes, (node) => {
      let typeIcon = "docker";
      if (node.title.startsWith("kubernetes_")) typeIcon = "kubernetes";
      // NOTE(nick): not all CoreOS objects will be prefixed with "coreos". The name
      // "CoreOS Fedora CoreOS" doesn't sounds great, right? We will probably need
      // another way to index "schema family" or something similar in the future.
      // For now, we only have one component, so let's use it.
      else if (node.title === "butane") typeIcon = "coreos";

      const componentQualificationSummary = _.find(
        qualificationSummary.value?.components,
        (cq) => cq.componentId.toString() === node.id,
      );
      let summaryStatus: QualificationStatus | undefined;
      if (componentQualificationSummary) {
        if (
          componentQualificationSummary.total >
          componentQualificationSummary.succeeded +
            componentQualificationSummary.failed
        )
          summaryStatus = "running";
        else if (componentQualificationSummary.failed > 0)
          summaryStatus = "failure";
        else summaryStatus = "success";
      }
      return {
        ...node,
        typeIcon,
        statusIcons: summaryStatus
          ? [qualificationStatusToIconMap[summaryStatus]]
          : [],
      };
    }),
  };
});

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

  await DiagramService2.actions.createConnection({
    fromNodeId,
    fromSocketId,
    toNodeId,
    toSocketId,
  });
}

async function onDiagramInsertElement(e: InsertElementEvent) {
  if (!lastInsertSelection.value)
    throw new Error("missing insert selection metadata");

  await DiagramService2.actions.createNode(
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
  DiagramService2.actions.updateNodePosition(e.id, e.position);
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
