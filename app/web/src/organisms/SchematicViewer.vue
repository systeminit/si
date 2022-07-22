<template>
  <div :id="viewerId" class="w-full h-full">
    <!-- We check for schematicData and schematicKind inside showViewer but typescript can't understand that so we check it here again -->
    <Viewer
      v-if="
        showViewer &&
        props.schematicKind &&
        filteredSchematicData &&
        editorContext
      "
      :schematic-viewer-id="viewerId"
      :viewer-state="viewerState"
      :editor-context="editorContext"
      :schematic-data="filteredSchematicData"
      :viewer-event$="props.viewerEvent$"
      :schematic-kind="props.schematicKind"
      :deployment-node-selected="props.deploymentNodeSelected"
    />
    <div
      v-else-if="props.schematicKind === SchematicKind.Component"
      class="flex place-content-center w-full h-full"
    >
      <div
        class="self-center w-2/3 h-2/3 border-2 border-gray-600 border-dashed rounded-md p-12 text-center hover:border-gray-500 justify-items-center"
      >
        <div
          class="flex-col text-center place-items-center items-center justify-items-center"
        >
          <h2 class="text-xl">Component Diagram</h2>
          <p class="text-xs">
            Contains a drill-down of the specific components that make up the
            currently selected node in the deployment diagram.
          </p>
          <p>&nbsp;</p>
          <p class="text-xs">
            Try selecting a node in the deployment diagram, and then adding a
            component to it.
          </p>
        </div>
      </div>
    </div>
    <div v-else class="flex place-content-center w-full h-full">
      <div
        class="self-center w-2/3 h-2/3 border-2 border-gray-600 border-dashed rounded-md p-12 text-center hover:border-gray-500 justify-items-center"
      >
        <div
          class="flex-col text-center place-items-center items-center justify-items-center"
        >
          <h2 class="text-xl">Deployment Diagram</h2>
          <p class="text-xs">
            Shows how the high level components of an application are deployed.
          </p>
          <p>&nbsp;</p>
          <p class="text-xs">
            For example, a Service deploys to a Kubernetes Cluster on AWS.
          </p>
          <p>&nbsp;</p>
          <p class="text-xs">Try adding a service to the diagram!</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import _ from "lodash";
import * as Rx from "rxjs";

import Viewer from "./SchematicViewer/Viewer.vue";

import { ViewerStateMachine } from "./SchematicViewer/state_machine";

import { refFrom, untilUnmounted } from "vuse-rx";
import { applicationNodeId$ } from "@/observable/application";
import { system$ } from "@/observable/system";
import { visibility$ } from "@/observable/visibility";
import {
  EditorContext,
  Schematic,
  SchematicKind,
} from "@/api/sdf/dal/schematic";
import { combineLatest, from } from "rxjs";
import { switchMap } from "rxjs/operators";
import { ViewerEvent } from "./SchematicViewer/viewer_event";
import { computed, ref } from "vue";

const props = defineProps<{
  viewerEvent$: Rx.ReplaySubject<ViewerEvent | null>;
  schematicData: Schematic | null;
  schematicKind: SchematicKind;
  deploymentNodeSelected: number | null;
  addingNode?: boolean;
}>();

const addingNode = ref(false);
visibility$.pipe(untilUnmounted).subscribe(() => {
  addingNode.value = false;
});

const filteredSchematicData = computed(() => {
  if (!props.schematicData) return undefined;

  const filteredSchematic: Schematic = {
    nodes: props.schematicData.nodes,
    connections: props.schematicData.connections,
  };
  const parentDeploymentNodeId = props.deploymentNodeSelected;

  // We want to ensure the nodes from the other panel are ignored
  // The deployment node also appears in the component panel
  // so we have to ignore it on the deployment panel
  filteredSchematic.nodes = filteredSchematic.nodes.filter(
    (node) =>
      (node.kind.kind === String(props.schematicKind) ||
        node.id === parentDeploymentNodeId) &&
      node.positions.length > 0,
  );

  // Find component nodes connected to selected deployment node
  const nodeIds = filteredSchematic.connections
    .filter((conn) => conn.destinationNodeId === parentDeploymentNodeId)
    .map((conn) => conn.sourceNodeId);

  if (parentDeploymentNodeId) {
    nodeIds.push(parentDeploymentNodeId);
  }

  switch (props.schematicKind) {
    case SchematicKind.Deployment:
      break;
    case SchematicKind.Component:
      // Filters component nodes that are children of selected deployment node
      filteredSchematic.nodes = filteredSchematic.nodes.filter((node) =>
        nodeIds.includes(node.id),
      );
      break;
  }

  // We need to remove connections from nodes that don't appear in our panel
  filteredSchematic.connections = filteredSchematic.connections.filter(
    (conn) => {
      return (
        filteredSchematic.nodes.find(
          (node) => node.id === conn.destinationNodeId,
        ) &&
        filteredSchematic.nodes.find((node) => node.id === conn.sourceNodeId)
      );
    },
  );
  return filteredSchematic;
});

const showViewer = computed(() => {
  if (
    !filteredSchematicData.value?.nodes?.length &&
    !(addingNode.value || props.addingNode)
  ) {
    return false;
  }

  if (props.schematicData && editorContext.value && props.schematicKind) {
    // Component panels pointing to a undefined deployment will sync selection with deployment panel
    // To avoid this we don't render a component panel pointing to a invalid deployment
    const isComponent = props.schematicKind === SchematicKind.Component;
    if (isComponent && props.deploymentNodeSelected === undefined) {
      return false;
    }
    return true;
  }
  return false;
});

const componentName = "SchematicViewer";
const componentId = _.uniqueId();

const viewerId = componentName + "-" + componentId;
const viewerState = new ViewerStateMachine();

const editorContext = refFrom<EditorContext | null>(
  combineLatest([system$, applicationNodeId$, visibility$]).pipe(
    switchMap(([system, applicationNodeId]) => {
      if (applicationNodeId) {
        return from([{ systemId: system?.id, applicationNodeId }]);
      } else {
        return from([null]);
      }
    }),
  ),
);
</script>

<style scoped>
.nodeadd-menu {
  width: 30%;
  margin-left: 35%;
  margin-top: 1em;
}
</style>
