<template>
  <Stack class="h-full w-full my-4 mx-4">
    <Inline align="center" alignY="bottom">
      <VormInput
        v-model="schemaVariant"
        label="Display"
        type="dropdown"
        class="flex-1"
        :options="schemaVariantOptions"
        placeholder="Only Diagram Components"
        placeholderSelectable
      />
      <VormInput
        v-model="search_query"
        label="Find Node"
        type="text"
        class="flex-1"
      />
      <div>
        <VButton
          :disabled="clickedNodes.size === 0 && clickedNeighbors.size === 0"
          @click="clearSelections"
          >Reset Selection(s)</VButton
        >
      </div>
      <div>
        <VButton :disabled="loading" @click="toggleLayout"
          >{{ animate_layout ? "Stop" : "Start" }} Animation</VButton
        >
      </div>
    </Inline>
    <h1 v-show="loading" align="center">Loading...</h1>
    <section
      v-show="!loading"
      id="vizDiv"
      class="h-full w-full m-0 p-0 overflow-hidden"
    ></section>
  </Stack>
</template>

<script lang="ts" setup>
import { onMounted, ref, computed, watchPostEffect, Ref, reactive } from "vue";
import { VormInput, Stack, Inline, VButton } from "@si/vue-lib/design-system";
import { DirectedGraph } from "graphology";
import Sigma from "sigma";
import { NodeDisplayData, EdgeDisplayData, Coordinates } from "sigma/types";
import FA2Layout from "graphology-layout-forceatlas2/worker";
import forceAtlas2 from "graphology-layout-forceatlas2";
import { useComponentsStore } from "@/store/components.store";
import { useVizStore } from "@/store/viz.store";

const vizStore = useVizStore();
const componentStore = useComponentsStore();

const loading: Ref<boolean> = ref(true);

const schemaVariant = ref();
const schemaVariantOptions = computed(() => {
  const options = componentStore.schemaVariants.map((sv) => ({
    label: sv.schemaName + (sv.builtin ? " (builtin)" : ""),
    value: sv.id,
  }));
  options.unshift({ label: "Full Workspace", value: "all" });
  return options;
});

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

const getColor = (nodeKind: string, contentKind: string | null) => {
  if (nodeKind === "Component") {
    return "#F0F";
  } else if (nodeKind === "Content") {
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
const animate_layout = ref(true);

// vanilla JS used for the graph library interactions
interface State {
  hoveredNode?: string;
  hoveredNeighbors?: Set<string>;

  clickedNodes: Set<string>;
  clickedNeighbors: Set<string>;

  // find a node
  selectedNode?: string;
  suggestions?: Set<string>;
}
// reactivity used for controls
const clickedNodes: Set<string> = reactive(new Set());
const clickedNeighbors: Set<string> = reactive(new Set());

const state: State = {
  clickedNodes,
  clickedNeighbors,
};
let graph: DirectedGraph;
let renderer: Sigma;
let fa2Layout: FA2Layout;

function toggleLayout() {
  if (fa2Layout.isRunning()) {
    fa2Layout.stop();
  } else {
    fa2Layout.start();
  }
  animate_layout.value = !animate_layout.value;
}

function buildClickedNeighbors() {
  state.clickedNeighbors.clear();
  state.clickedNodes.forEach((c: string) => {
    for (const n of graph.neighbors(c)) {
      state.clickedNeighbors.add(n);
    }
  });
}

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

function clearSelections() {
  state.clickedNodes.clear();
  state.clickedNeighbors.clear();
}

// we need DOM loaded
onMounted(async () => {
  const size: Ref<number> = ref(3);

  // whenever the schema variant changes, we need to re-load the graph
  // `Post` because we need the DOM loaded
  watchPostEffect(async (): Promise<void> => {
    if (renderer) {
      renderer.kill(); // without this the graph draws new nodes on top of old nodes
    }

    size.value = 6;
    if (schemaVariant.value === "all") {
      size.value = 3; // large graph, smaller dots
    }

    loading.value = true;
    if (schemaVariant.value) {
      const variant =
        schemaVariant.value !== "all" ? schemaVariant.value : null;
      await vizStore.LOAD_VARIANTS(variant);
    } else {
      await vizStore.LOAD_COMPONENTS();
    }
    loading.value = false;

    if (vizStore.edges.length === 0 && vizStore.nodes.length === 0) {
      return;
    }

    // build graph nodes
    graph = new DirectedGraph();

    for (const node of vizStore.nodes) {
      graph.addNode(node.id, {
        color: getColor(node.nodeKind, node.contentKind),
        label: `${node.contentKind ?? node.nodeKind}${
          node.name ? `: ${node.name}` : ""
        }`,
        x: Math.floor(Math.random() * 1000),
        y: Math.floor(Math.random() * 1000),
        size: node.nodeKind === "Component" ? size.value * 2 : size.value, // make it stand out
        grouping:
          node.nodeKind === "Content" ? node.contentKind : node.contentKind,
      });
    }

    for (const edge of vizStore.edges) {
      graph.addEdge(edge.from, edge.to);
    }

    const container = document.getElementById("vizDiv") as HTMLElement;
    renderer = new Sigma(graph, container, {
      allowInvalidContainer: true,
    });

    const sensibleSettings = forceAtlas2.inferSettings(graph);
    fa2Layout = new FA2Layout(graph, { settings: sensibleSettings });
    fa2Layout.start();
    animate_layout.value = true; // override, since we're re-drawing the graph

    // Cheap trick: tilt the camera a bit to make labels more readable
    renderer.getCamera().setState({
      angle: 0.2,
    });

    // Define graph interactivity
    renderer.on("enterNode", ({ node }) => {
      setHoveredNode(node);
    });
    renderer.on("leaveNode", () => {
      setHoveredNode(undefined);
    });

    renderer.on("clickNode", ({ node }) => {
      if (state.clickedNodes.has(node)) {
        state.clickedNodes.delete(node);
      } else {
        state.clickedNodes.add(node);
      }
      buildClickedNeighbors();
    });

    renderer.setSetting("nodeReducer", (node, data) => {
      const res: Partial<NodeDisplayData> = { ...data };
      let dim = false;

      if (
        state.hoveredNeighbors &&
        !state.hoveredNeighbors.has(node) &&
        state.hoveredNode !== node
      ) {
        dim = true;
      }

      if (
        state.clickedNeighbors.size > 0 &&
        !state.clickedNeighbors.has(node)
      ) {
        dim = true;
      }

      if (state.suggestions && !state.suggestions.has(node)) {
        dim = true;
      }

      if (state.selectedNode === node) {
        res.highlighted = true; // displays the label
        dim = false;
      }

      if (state.clickedNodes.has(node)) {
        res.highlighted = true;
        dim = false;
      }

      if (
        state.clickedNeighbors.has(node) ||
        state.suggestions?.has(node) ||
        state.hoveredNeighbors?.has(node)
      ) {
        dim = false;
      }

      if (dim) {
        res.color = "#f6f6f6";
      }

      return res;
    });

    renderer.setSetting("edgeReducer", (edge, data) => {
      const res: Partial<EdgeDisplayData> = { ...data };
      const source = graph.source(edge);
      const target = graph.target(edge);

      if (state.hoveredNode && !graph.hasExtremity(edge, state.hoveredNode)) {
        res.hidden = true;
      }

      if (
        state.suggestions &&
        (!state.suggestions.has(source) || !state.suggestions.has(target))
      ) {
        res.hidden = true;
      }

      if (
        state.clickedNeighbors.has(source) ||
        state.clickedNeighbors.has(target) ||
        state.clickedNodes.has(source) ||
        state.clickedNodes.has(target)
      ) {
        res.hidden = false;
      }

      return res;
    });
  });

  // search reactivity needs manual intervention for the graph library
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

    renderer.refresh(); // edgeReducer doesn't run without this
  });
});
</script>
