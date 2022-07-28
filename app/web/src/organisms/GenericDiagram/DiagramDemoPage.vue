/** TEMPORARY demo page - route accessible at /diagram */
<template>
  <div class="bg-action-500 w-full h-full overflow-hidden flex flex-col">
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
        @nodeMove="onNodeMove"
        @drawEdge="onDrawEdge"
        @deleteElements="onDelete"
        @insertElement="onInsert"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { reactive, ref, watch } from "vue";
import _ from "lodash";
import {
  DeleteEvent,
  DiagramEdgeDef,
  DiagramNodeDef,
  DiagramSocketDef,
  DrawEdgeEvent,
  InsertElementEvent,
  NodeMoveEvent,
} from "./diagram_types";
import GenericDiagram from "./GenericDiagram.vue";

const ZOOM_OPTIONS = [25, 50, 75, 100, 125, 150, 200, 300, 400, 500];
const zoom = ref(1);
watch(zoom, () => diagramRef?.value?.setZoom(zoom.value));

const diagramRef = ref<InstanceType<typeof GenericDiagram>>();

const getSockets = (nodeId: string): DiagramSocketDef[] => [
  {
    id: `${nodeId}/str-in-1`,
    label: "string 1 input",
    nodeSide: "left",
    maxConnections: 1,
    type: "string",
    direction: "input",
  },
  {
    id: `${nodeId}/str-in-2`,
    label: "string 2 input",
    nodeSide: "left",
    maxConnections: 1,
    type: "string",
    direction: "input",
  },
  {
    id: `${nodeId}/num-in-1`,
    label: "number 1 input",
    nodeSide: "left",
    maxConnections: 1,
    type: "number",
    direction: "input",
  },
  {
    id: `${nodeId}/str-out`,
    label: "cool string output",
    nodeSide: "right",
    maxConnections: null,
    type: "string",
    direction: "output",
  },
  {
    id: `${nodeId}/num-out`,
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
    title: "Node 1",
    subtitle: "si-n1",
    type: "regular",
    position: { x: 0, y: 0 },
    sockets: getSockets("n1"),
    color: "#2F80ED",
  },
  {
    id: "n2",
    title: "Node 2 has a super duper long name!",
    subtitle: "si-n2",
    type: "regular",
    position: { x: 250, y: 0 },
    sockets: getSockets("n2"),
    color: "#A752DE",
  },
  {
    id: "n3",
    title: "Node 3",
    subtitle: "si-n3",
    type: "regular",
    position: { x: 250, y: 150 },
    sockets: getSockets("n3"),
    color: "#C23E7F",
  },
  {
    id: "n4",
    title: "Node 4",
    // subtitle: "si-n4",
    type: "regular",
    position: { x: 500, y: 0 },
    sockets: getSockets("n4"),
    color: "#5AACAD",
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

function onNodeMove(e: NodeMoveEvent) {
  const movedNode = nodes.find((n) => n.id === e.id);
  if (!movedNode) return;
  movedNode.position = e.position;
}
function onDrawEdge(e: DrawEdgeEvent) {
  edges.push({
    fromSocketId: e.fromSocketId,
    toSocketId: e.toSocketId,
    id: `${e.fromNodeId}/${_.uniqueId()}`,
  });
}
function onDelete(e: DeleteEvent) {
  _.each(e.elements, (el) => {
    if (el.diagramElementType === "node") {
      nodes.splice(_.findIndex(nodes, { id: el.id }), 1);
    } else if (el.diagramElementType === "edge") {
      edges.splice(_.findIndex(edges, { id: el.id }), 1);
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
        title: `Node ${newNodeId}`,
        subtitle: `si-${newNodeId}`,
        type: "regular",
        position: e.position,
        sockets: getSockets(`n${newNodeId}`),
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
