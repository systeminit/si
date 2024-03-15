<template>
  <Stack class="h-full w-full">
    <section>
      <VormInput
        v-model="search_query"
        label="Find Node"
        type="text"
        class="flex-1"
      />
    </section>
    <section
      id="vizDiv"
      class="h-full w-full m-0 p-0 overflow-hidden"
    ></section>
  </Stack>
</template>

<script lang="ts" setup>
import { VormInput, Stack } from "@si/vue-lib/design-system";
import { DirectedGraph } from "graphology";
import Sigma from "sigma";
import { NodeDisplayData, EdgeDisplayData, Coordinates } from "sigma/types";
import FA2Layout from "graphology-layout-forceatlas2/worker";
import forceAtlas2 from "graphology-layout-forceatlas2";
import { onMounted, ref, watchPostEffect } from "vue";
import { useVizStore } from "@/store/viz.store";

const vizStore = useVizStore();

const props = defineProps<{ schemaVariantId?: string }>();

const getColor = (nodeKind: string, contentKind: string | null) => {
  const kindMap: { [key: string]: string } = {
    Category: "#c200ff",
    Func: "#F02",
    Ordering: "#ABC",
    Prop: "#0FF",
    AttributePrototypeArgument: "#103016",
    FuncArgument: "#40300c",
  };

  const contentKindMap: { [key: string]: string } = {
    Root: "#F0F",
    ActionPrototype: "#FFA",
    AttributePrototype: "#00F",
    InputSocket: "#07A",
    OutputSocket: "#07A",
    Schema: "#0F0",
    FuncArg: "#777",
  };

  if (nodeKind === "Content") {
    if (contentKind) {
      return contentKindMap[contentKind] ?? "#AF0";
    } else {
      return "#AF0";
    }
  } else {
    return kindMap[nodeKind] ?? "#AF0";
  }
};

const search_query = ref("");

interface State {
  hoveredNode?: string;

  // State derived from query:
  selectedNode?: string;
  suggestions?: Set<string>;

  // State derived from hovered node:
  hoveredNeighbors?: Set<string>;
}
const state: State = {};

onMounted(async () => {
  let nodesAndEdges;
  let size: number;
  if (!props.schemaVariantId) {
    nodesAndEdges = await vizStore.FETCH_VIZ();
    size = 2;
  } else {
    nodesAndEdges = await vizStore.FETCH_SCHEMA_VARIANT_VIZ(
      props.schemaVariantId,
    );
    size = 6;
  }

  if (!nodesAndEdges.result.success) {
    return;
  }

  const nodes = nodesAndEdges.result.data.nodes;
  const edges = nodesAndEdges.result.data.edges;

  const graph = new DirectedGraph();

  for (const node of nodes) {
    graph.addNode(node.id, {
      color: getColor(node.nodeKind, node.contentKind),
      label: `${node.contentKind ?? node.nodeKind}${
        node.name ? `: ${node.name}` : ""
      }`,
      x: Math.floor(Math.random() * 1000),
      y: Math.floor(Math.random() * 1000),
      size,
    });
  }

  for (const edge of edges) {
    graph.addEdge(edge.from, edge.to);
  }

  const sensibleSettings = forceAtlas2.inferSettings(graph);
  const fa2Layout = new FA2Layout(graph, { settings: sensibleSettings });

  fa2Layout.start();

  const container = document.getElementById("vizDiv") as HTMLElement;
  const renderer = new Sigma(graph, container, { allowInvalidContainer: true });

  renderer.on("enterNode", ({ node }) => {
    setHoveredNode(node);
  });
  renderer.on("leaveNode", () => {
    setHoveredNode(undefined);
  });

  function setHoveredNode(node?: string) {
    if (node) {
      state.hoveredNode = node;
      state.hoveredNeighbors = new Set(graph.neighbors(node));
    } else {
      state.hoveredNode = undefined;
      state.hoveredNeighbors = undefined;
    }

    renderer.refresh();
  }

  renderer.setSetting("nodeReducer", (node, data) => {
    const res: Partial<NodeDisplayData> = { ...data };

    if (
      state.hoveredNeighbors &&
      !state.hoveredNeighbors.has(node) &&
      state.hoveredNode !== node
    ) {
      res.label = "";
      res.color = "#f6f6f6";
    }

    if (state.selectedNode === node) {
      res.highlighted = true;
    } else if (state.suggestions && !state.suggestions.has(node)) {
      res.label = "";
      res.color = "#f6f6f6";
    }

    return res;
  });

  renderer.setSetting("edgeReducer", (edge, data) => {
    const res: Partial<EdgeDisplayData> = { ...data };

    if (state.hoveredNode && !graph.hasExtremity(edge, state.hoveredNode)) {
      res.hidden = true;
    }

    if (
      state.suggestions &&
      (!state.suggestions.has(graph.source(edge)) ||
        !state.suggestions.has(graph.target(edge)))
    ) {
      res.hidden = true;
    }

    return res;
  });

  watchPostEffect((): void => {
    if (!search_query.value) {
      state.selectedNode = undefined;
      state.suggestions = undefined;
      return;
    }

    const q = search_query.value.toLowerCase();
    const options = graph
      .nodes()
      .map((n) => ({
        id: n,
        label: graph.getNodeAttribute(n, "label") as string,
      }))
      .filter((data) => data.label.toLowerCase().includes(q));

    if (options.length === 1 && options[0]?.label === search_query.value) {
      state.selectedNode = options[0].id;
      state.suggestions = undefined;

      // Move the camera to center it on the selected node:
      const nodePosition = renderer.getNodeDisplayData(
        state.selectedNode,
      ) as Coordinates;
      renderer.getCamera().animate(nodePosition, {
        duration: 500,
      });
    } else {
      state.selectedNode = undefined;
      state.suggestions = new Set(options.map(({ id }) => id));
    }
  });
});
</script>
