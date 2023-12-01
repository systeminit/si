/** TEMPORARY demo page - route accessible at /diagram */
<template>
  <div class="bg-action-500 w-full h-screen overflow-hidden flex flex-col">
    <div class="text-white bg-black w-full h-10 z-10 flex space-x-10">
      <select v-model="zoom" class="text-black">
        <option
          v-for="zoomLevel in ZOOM_OPTIONS"
          :key="`zoom-${zoomLevel}`"
          :value="zoomLevel / 100"
        >
          {{ zoomLevel }}
        </option>
      </select>
      <select v-model="theme" class="text-black">
        <option>dark</option>
        <option>light</option>
      </select>

      <div>zoom = {{ zoom }}</div>

      <a href="#" @click.prevent="diagramRef?.recenter">recenter</a>
      <a href="#" @click.prevent="diagramRef?.beginInsertElement('node')"
        >add node</a
      >
    </div>
    <div class="relative flex-grow">
      <GenericDiagram
        ref="diagramRef"
        :nodes="nodes"
        :edges="edges"
        @update:zoom="onUpdateZoom"
        @move-element="onNodeMove"
        @delete-elements="onDelete"
        @insert-element="onInsert"
        @draw-edge="onDrawEdge"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { reactive, ref, watch } from "vue";
import * as _ from "lodash-es";
import {
  useThemeContainer,
  ThemeValue,
  COLOR_PALETTE,
} from "@si/vue-lib/design-system";
import GenericDiagram from "./GenericDiagram.vue";
import {
  DeleteElementsEvent,
  DiagramEdgeData,
  DiagramEdgeDef,
  DiagramNodeData,
  DiagramNodeDef,
  DiagramSocketDef,
  DrawEdgeEvent,
  InsertElementEvent,
  MoveElementEvent,
} from "./diagram_types";

const ZOOM_OPTIONS = [25, 50, 75, 100, 125, 150, 200, 300, 400, 500];
const zoom = ref(1);
watch(zoom, () => diagramRef?.value?.setZoom(zoom.value));

const theme = ref<ThemeValue>("dark");
useThemeContainer(theme.value);

const diagramRef = ref<InstanceType<typeof GenericDiagram>>();

const getSockets = (_nodeId: string): DiagramSocketDef[] => [
  {
    id: `str-in-1`,
    label: "string 1 input",
    nodeSide: "left",
    maxConnections: 1,
    type: "string",
    direction: "input",
  },
  {
    id: `str-in-2`,
    label: "string 2 input",
    nodeSide: "left",
    maxConnections: 1,
    type: "string",
    direction: "input",
  },
  {
    id: `num-in-1`,
    label: "number 1 input",
    nodeSide: "left",
    maxConnections: 1,
    type: "number",
    direction: "input",
  },
  {
    id: `str-out`,
    label: "cool string output",
    nodeSide: "right",
    maxConnections: null,
    type: "string",
    direction: "output",
  },
  {
    id: `num-out`,
    label: "amazing number output",
    nodeSide: "right",
    maxConnections: null,
    type: "number",
    direction: "output",
  },
];

const nodes = reactive<DiagramNodeDef[]>([
  {
    id: "n1",
    componentId: "c1",
    title: "Node 1",
    subtitle: "si-n1",
    type: "regular",
    position: { x: 0, y: 0 },
    sockets: getSockets("n1"),
    color: COLOR_PALETTE.action[500],
    typeIcon: "logo-docker",
    isLoading: false,
    nodeType: "component",
    changeStatus: "unmodified",
  },
  {
    id: "n2",
    componentId: "c2",
    title: "Node 2 has a super duper long name!",
    subtitle: "si-n2",
    type: "regular",
    position: { x: 250, y: 0 },
    sockets: getSockets("n2"),
    color: "#A752DE",
    typeIcon: "logo-docker",
    isLoading: false,
    nodeType: "component",
    changeStatus: "unmodified",
  },
  {
    id: "n3",
    componentId: "c3",
    title: "Node 3",
    subtitle: "si-n3",
    type: "regular",
    position: { x: 250, y: 200 },
    sockets: getSockets("n3"),
    color: "#C23E7F",
    typeIcon: "logo-k8s",
    statusIcons: [
      { icon: "check-square", tone: "success" },
      { icon: "alert-circle", tone: "warning" },
      { icon: "x-circle", tone: "error" },
      { icon: "loader", tone: "info" },
      // { icon: "logo-docker", tone: "neutral" },
    ],
    isLoading: false,
    nodeType: "component",
    changeStatus: "unmodified",
  },
  {
    id: "n4",
    componentId: "c4",
    title: "Node 4",
    // subtitle: "si-n4",
    type: "regular",
    position: { x: 500, y: 0 },
    sockets: getSockets("n4"),
    color: "#5AACAD",
    typeIcon: "logo-k8s",
    isLoading: false,
    nodeType: "component",
    changeStatus: "unmodified",
  },
  {
    id: "n5",
    componentId: "c5",
    title: "Node 5",
    subtitle: "is being operated on!",
    type: "regular",
    position: { x: 500, y: 200 },
    sockets: getSockets("n4"),
    color: "#FF9900",
    typeIcon: "logo-k8s",
    isLoading: true,
    nodeType: "component",
    changeStatus: "unmodified",
  },
]);

// update node title to make it longer and check if height responds correctly
// setInterval(() => {
//   nodes[1].title += "x";
// }, 200);

const edges = reactive<DiagramEdgeDef[]>([
  // {
  //   id: "e1",
  //   fromSocketId: "n1/out",
  //   toSocketId: "n2/in",
  //   isBidirectional: true,
  // },
  // { id: "e2", fromSocketId: "n2/out", toSocketId: "n3/in" },
  // { id: "e3", fromSocketId: "n2/out", toSocketId: "n4/in" },
]);

function onNodeMove(e: MoveElementEvent) {
  if (e.element instanceof DiagramNodeData) {
    const nodeId = e.element.def.id;
    const movedNode = nodes.find((n) => n.id === nodeId);
    if (!movedNode) return;
    movedNode.position = e.position;
  }
}

function onDrawEdge(e: DrawEdgeEvent) {
  edges.push({
    fromNodeId: e.fromSocket.parent.def.id,
    fromSocketId: e.fromSocket.def.id,
    toNodeId: e.toSocket.parent.def.id,
    toSocketId: e.toSocket.def.id,
    id: `${e.fromSocket.def.id}/${_.uniqueId()}`,
    changeStatus: "unmodified",
  });
}

function onDelete(e: DeleteElementsEvent) {
  _.each(e.elements, (el) => {
    if (el instanceof DiagramNodeData) {
      nodes.splice(_.findIndex(nodes, { id: el.def.id }), 1);
    } else if (el instanceof DiagramEdgeData) {
      edges.splice(_.findIndex(edges, { id: el.def.id }), 1);
    }
  });
}

function onInsert(e: InsertElementEvent) {
  if (e.diagramElementType === "node") {
    const newNodeId = +new Date();
    // hacky example just to show adding delay and how we'll tell the diagram we're done inserting
    setTimeout(() => {
      nodes.push({
        id: `n${newNodeId}`,
        componentId: `c${newNodeId}`,
        title: `Node ${newNodeId}`,
        subtitle: `si-${newNodeId}`,
        type: "regular",
        position: e.position,
        sockets: getSockets(`n${newNodeId}`),
        typeIcon: "logo-docker",
        isLoading: false,
        nodeType: "component",
        changeStatus: "unmodified",
      });

      // parent needs to call this when insert is complete
      e.onComplete();
    }, 1500);
  }
}

function onUpdateZoom(newZoom: number) {
  zoom.value = newZoom;
}
</script>
