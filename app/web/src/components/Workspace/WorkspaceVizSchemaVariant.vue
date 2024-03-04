<template>
  <div id="vizDiv" class="h-full w-full m-0 p-0 overflow-hidden"></div>
</template>

<script lang="ts" setup>
import { onMounted } from "vue";
import { DirectedGraph } from "graphology";
import Sigma from "sigma";
import FA2Layout from "graphology-layout-forceatlas2/worker";
import forceAtlas2 from "graphology-layout-forceatlas2";
import { useVizStore } from "@/store/viz.store";

const vizStore = useVizStore();

const props = defineProps<{ schemaVariantId?: string }>();

const getColor = (nodeKind: string, contentKind: string | null) => {
  const kindMap: { [key: string]: string } = {
    Category: "#2FA",
    Func: "#F02",
    Ordering: "#ABC",
    Prop: "#0FF",
  };

  const contentKindMap: { [key: string]: string } = {
    Root: "#F0F",
    ActionPrototype: "#FFA",
    AttributePrototype: "#00F",
    InternalProvider: "#07A",
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

  const graph = new DirectedGraph();

  const nodes = nodesAndEdges.result.data.nodes;
  const edges = nodesAndEdges.result.data.edges;

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
  const fa2Layout = new FA2Layout(graph, {
    settings: sensibleSettings,
  });

  fa2Layout.start();

  const container = document.getElementById("vizDiv") as HTMLElement;
  // tslint:disable-next-line
  const _sigma = new Sigma(graph, container);
});
</script>
