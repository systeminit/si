<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <section id="map" class="grid h-full">
    <div
      v-if="selectedComponent"
      id="selection"
      class="absolute top-[110px] max-w-[350px] min-w-[300px] right-3 p-xs bg-action-800 border-2 border-action-700 scrollable"
      style="max-height: calc(100vh - 115px)"
    >
      <!-- NOTE: 110 is the fixed size (nothing expands/dynamic) above, extra 5px on the bottom, unforuntately tailwind doesn't support that calc -->
      <ComponentGridTile
        :component="selectedComponent"
        hideConnections
        @click="componentNavigate(selectedComponent.id)"
      />
      <ConnectionsPanel :component="selectedComponent" />
    </div>

    <div
      id="controls"
      class="absolute left-0 bottom-0 flex flex-row gap-sm m-sm items-center"
    >
      <div class="text-white">{{ Math.round(zoomLevel * 100) }}%</div>
      <div
        v-tooltip="'Zoom Out'"
        :class="getButtonClasses(zoomLevel >= MAX_ZOOM)"
        @click="zoomOut"
      >
        <Icon name="minus" size="sm" />
      </div>
      <div
        v-tooltip="'Zoom In'"
        :class="getButtonClasses(zoomLevel >= MAX_ZOOM)"
        @click="zoomIn"
      >
        <Icon name="plus" size="sm" />
      </div>
      <div v-tooltip="'Reset'" :class="getButtonClasses(false)" @click="reset">
        <Icon name="empty-square" size="sm" />
      </div>
    </div>

    <svg
      height="100%"
      width="100%"
      preserveAspectRatio="xMidYMid"
      :viewBox="`0 0 ${viewBox} ${viewBox}`"
      @wheel="wheel"
      @mousedown.left.prevent="mousedown"
      @mouseup.left="mouseup"
      @mousemove="mousemove"
    >
      <defs v-for="logo in logos" :key="logo">
        <pattern
          :id="`${logo}`"
          width="1"
          height="1"
          patternUnits="objectBoundingBox"
        >
          <IconNoWrapper :name="logo" class="text-white" />
        </pattern>
      </defs>
      <template v-for="icon in icons" :key="icon">
        <defs v-for="tone in tones" :key="tone">
          <pattern
            :id="`${icon}-${tone}`"
            width="30"
            height="30"
            patternUnits="objectBoundingBox"
          >
            <IconNoWrapper :name="icon" :tone="tone" size="sm" />
          </pattern>
        </defs>
      </template>
      <g :transform="`matrix(${transformMatrix})`"></g>
      <circle :cx="origCenter" :cy="origCenter" r="5" fill="yellow" />
    </svg>

    <ComponentContextMenu
      ref="componentContextMenuRef"
      :viewId="viewId"
      :componentIds="selectedComponent ? [selectedComponent.id] : []"
    />
  </section>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import {
  IconNoWrapper,
  Icon,
  IconNames,
  Tones,
  themeClasses,
  LOGO_ICONS,
} from "@si/vue-lib/design-system";
import {
  computed,
  ComputedRef,
  inject,
  nextTick,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  unref,
  watch,
} from "vue";
import ELK from "elkjs/lib/elk.bundled.js";
import * as d3 from "d3";
import clsx from "clsx";
import { tw } from "@si/vue-lib";
import { useRoute, useRouter } from "vue-router";
import * as _ from "lodash-es";
import { Fzf } from "fzf";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  BifrostComponent,
  BifrostComponentConnections,
  BifrostIncomingConnectionsList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { SelectionsInQueryString } from "./Workspace.vue";
import { KeyDetails, keyEmitter } from "./logic_composables/emitters";
import { assertIsDefined, Context } from "./types";
import ComponentGridTile from "./ComponentGridTile.vue";
import ConnectionsPanel from "./ConnectionsPanel.vue";
import { getAssetIcon } from "./util";
import ComponentContextMenu from "./ComponentContextMenu.vue";

const props = defineProps<{
  active: boolean;
  viewId: string;
}>();

const componentContextMenuRef =
  ref<InstanceType<typeof ComponentContextMenu>>();

const selectedComponent = ref<BifrostComponent | null>(null);

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

// don't change this!
// magic number puts the yellow dot in the middle!
// driving me a bit insane
const viewBox = ref(1500);
const origCenter = ref(viewBox.value / 2);
const origTransformMatrix = [1, 0, 0, 1, 0, 0];
const transformMatrix = reactive<number[]>([...origTransformMatrix]);

const scaleStep = 0.13;
const baseZoom = 1;
const zoomLevel = ref(baseZoom);
const lastPos = ref<xy | null>(null);
const MAX_ZOOM = 5;
const MIN_ZOOM = 0.2;

const applyZoom = () => {
  zoomLevel.value = Math.fround(zoomLevel.value);
  zoomLevel.value = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoomLevel.value));

  // Zoom implementation
  // Attempt 1
  // "FPS style" - always zoom to the center point of the SVG
  // move what you want to the center to see it

  // first and fourth elements of the matrix describe
  // the X & Y scale (aka zoom) of the coordinate space
  const scaleX = transformMatrix[0] ?? 1;
  const scaleY = transformMatrix[3] ?? 1;

  // the last two are X & Y adjustments to origin 0,0
  // 0,0 is top left, positive values move down and right
  // negative values move up and left
  // panning the stage changes these values without changing scale
  const translateX = transformMatrix[4] ?? 0;
  const translateY = transformMatrix[5] ?? 0;

  /** find the center of the SVG coordinate space
   * as we scale the coordinate space we need to adjust
   * what "center" is e.g. zooming in, space between points
   * grows which moves the center point down and right
   * and so the top left must move negatively to keep
   * the center point _in_ the center of the screen
   */
  const svgCenterX = origCenter.value;
  const svgCenterY = origCenter.value;

  /** get the difference between "center" and where
   * we have moved the origin off 0,0, from both panning and zoom
   * and divide by the current scale. this normalizes us back to
   * an "unscaled" coordinate space
   */
  const worldX = (svgCenterX - translateX) / scaleX;
  const worldY = (svgCenterY - translateY) / scaleY;

  const newScale = zoomLevel.value;

  // now translate the new points against the new zoom level
  // to get the proper X, Y panning adjustments for the grown/shrunk
  // coordinate space
  const newTranslateX = svgCenterX - worldX * newScale;
  const newTranslateY = svgCenterY - worldY * newScale;

  transformMatrix.splice(
    0,
    6,
    newScale,
    0,
    0,
    newScale,
    newTranslateX,
    newTranslateY,
  );
};

const pan = (dx: number, dy: number) => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const x = transformMatrix[4]!;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const y = transformMatrix[5]!;
  transformMatrix.splice(4, 1, x + dx);
  transformMatrix.splice(5, 1, y + dy);
};

const mouseDown = ref(false);

const zoomIn = () => {
  zoomLevel.value += scaleStep;
  applyZoom();
};

const zoomOut = () => {
  zoomLevel.value -= scaleStep;
  applyZoom();
};

const wheel = (event: WheelEvent) => {
  if (event.metaKey || event.ctrlKey) {
    // Zoom behavior when modifier key is held
    if (event.deltaY > 0) {
      zoomOut();
    } else {
      zoomIn();
    }
  } else {
    // Pan behavior - move map in scroll direction
    const panSpeed = 40;
    pan((-event.deltaX * panSpeed) / 100, (-event.deltaY * panSpeed) / 100);
  }
};

const reset = () => {
  zoomLevel.value = 1;
  transformMatrix.splice(0, 6, 1, 0, 0, 1, 0, 0);
};

const mousedown = () => {
  // NOTE: the .prevent on the handler above stops text selection within the svg
  mouseDown.value = true;
};

const mouseup = () => {
  mouseDown.value = false;
};

const active = computed(() => props.active);

const KEYSTEP = 10;
const onArrowDown = () => {
  if (!active.value) return;
  pan(0, -KEYSTEP);
};
const onArrowUp = () => {
  if (!active.value) return;
  pan(0, KEYSTEP);
};
const onArrowLeft = () => {
  if (!active.value) return;
  pan(KEYSTEP, 0);
};
const onArrowRight = () => {
  if (!active.value) return;
  pan(-KEYSTEP, 0);
};
const onEscape = () => {
  if (!active.value) return;
  selectedComponent.value = null;
};
const onE = (e: KeyDetails["e"]) => {
  if (selectedComponent.value && (e.ctrlKey || e.metaKey)) {
    componentContextMenuRef.value?.componentsStartErase([
      selectedComponent.value.id,
    ]);
  }
};
const mountKeyEmitters = () => {
  keyEmitter.on("ArrowDown", onArrowDown);
  keyEmitter.on("ArrowUp", onArrowUp);
  keyEmitter.on("ArrowLeft", onArrowLeft);
  keyEmitter.on("ArrowRight", onArrowRight);
  keyEmitter.on("Escape", onEscape);
  keyEmitter.on("e", onE);
};
const removeKeyEmitters = () => {
  keyEmitter.off("ArrowDown", onArrowDown);
  keyEmitter.off("ArrowUp", onArrowUp);
  keyEmitter.off("ArrowLeft", onArrowLeft);
  keyEmitter.off("ArrowRight", onArrowRight);
  keyEmitter.off("Escape", onEscape);
  keyEmitter.off("e", onE);
};
onMounted(() => {
  // if we need to adjust zoom level on load dynamically
  // change it here
  applyZoom();

  mountKeyEmitters();
});
onBeforeUnmount(() => {
  removeKeyEmitters();
});

const mousemove = (event: MouseEvent) => {
  const current: xy = { x: event.clientX, y: event.clientY };
  if (!mouseDown.value || !lastPos.value) {
    lastPos.value = { ...current };
    return;
  }
  const diff: xy = {
    x: current.x - lastPos.value.x,
    y: current.y - lastPos.value.y,
  };
  lastPos.value = { ...current };
  pan(diff.x, diff.y);
};

const key = useMakeKey();
const args = useMakeArgs();

const queryKey = key(EntityKind.IncomingConnectionsList);

const logos = reactive<IconNames[]>(Object.keys(LOGO_ICONS) as IconNames[]);

const icons = reactive<IconNames[]>([
  "check-hex-outline",
  "check-hex",
  "x-hex-outline",
]);
const tones = reactive<Tones[]>(["success", "destructive"]);

const connections = useQuery<BifrostIncomingConnectionsList>({
  queryKey,
  queryFn: async () => {
    const d = await bifrost<BifrostIncomingConnectionsList | null>(
      args(EntityKind.IncomingConnectionsList),
    );

    if (d) {
      // this sets the component from the URL querystring on load, and then doesn't re-enter
      if (fillDefault.value) {
        nextTick(() => {
          const c = d.componentConnections.find(
            (c) => c.id === fillDefault.value,
          );
          if (c && c.component) selectedComponent.value = c.component;
          fillDefault.value = undefined;
        });
      }

      return d;
    } else
      return {
        id: unref(ctx.changeSetId),
        componentConnections: [] as BifrostComponentConnections[],
      };
  },
});

const mapData = computed(() => {
  const nodes = new Set<string>();
  const edges = new Set<string>();
  const components: Record<string, BifrostComponent> = {};
  if (!connections.data.value) {
    return { nodes, edges, components };
  }

  const matchingIds: string[] = [];
  if (searchString?.value && searchString.value.trim().length > 0) {
    const componentsMap: Record<string, BifrostComponent> = {};
    connections.data.value.componentConnections.forEach((c) => {
      componentsMap[c.id] = c.component;
    });

    const fzf = new Fzf(Object.values(componentsMap), {
      casing: "case-insensitive",
      selector: (c) =>
        `${c.name} ${c.schemaVariantName} ${c.schemaName} ${c.schemaCategory} ${c.schemaId} ${c.id}`,
    });

    const results = fzf.find(searchString.value);
    if (results.length === 0) return { nodes, edges, components };
    else matchingIds.push(...results.map((c) => c.item.id));
  }

  connections.data.value.componentConnections.forEach((c) => {
    if (searchString?.value && !matchingIds.includes(c.id)) return;

    nodes.add(c.id);
    components[c.id] = c.component;
    c.incoming.forEach((e) => {
      // incoming, so "to" is me, always start with "me"
      if (
        searchString?.value &&
        (!matchingIds.includes(e.toComponent.id) ||
          !matchingIds.includes(e.fromComponent.id))
      )
        return;

      // TODO(nick): found this... technically isn't possible anymore, but I'm leaving until we get
      // weak references working... Original comment continues below:
      //
      // in case of problems with the data, filter out undefined
      // if they're left in the graph won't render
      if (!e.toComponent.id || !e.fromComponent.id) return;

      const edge = `${e.toComponent.id}-${e.fromComponent.id}`;
      edges.add(edge);
    });
  });

  return { nodes, edges, components };
});

type node = {
  id: string;
  width: number;
  height: number;
  component: BifrostComponent;
  icons: [string | null];
};

type xy = {
  x: number;
  y: number;
};

type h = {
  $H: number;
};

type edge = {
  id: string;
  sources: string[];
  targets: string[];
};

type line = {
  sections: {
    id: string;
    startPoint: xy;
    endPoint: xy;
    bendPoints?: xy[];
    incomingShape: string;
    outgoingShape: string;
  }[];
  container: string;
};

type layoutNode = node & xy & h;
type layoutLine = line & edge;

const dataAsGraph = ref<unknown>();

const WIDTH = 250;
const HEIGHT = 75;

const router = useRouter();
const clickedNode = (e: MouseEvent, n: layoutNode) => {
  e.preventDefault();
  if (selectedComponent.value?.id === n.component.id) {
    selectedComponent.value = null;
    componentContextMenuRef.value?.close();
  } else {
    selectedComponent.value = n.component;
    // TODO - figure out menu placement here!
    // componentContextMenuRef.value?.open(IDK, [n.component.id]);
  }
};

watch(selectedComponent, () => {
  // this handles later changes after the page loads
  document
    .querySelectorAll("#map > svg rect.node")
    .forEach((n) => n.classList.remove("selected"));
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.c;
  if (selectedComponent.value) {
    query.c = selectedComponent.value.id;
    document
      .querySelector(`#map > svg rect.node.id-${selectedComponent.value.id}`)
      ?.classList.add("selected");
  }
  router.push({ query });
});

const searchString = inject<ComputedRef<string>>("SEARCH");

const fillDefault = ref<string>();
onMounted(() => {
  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  if (query.c) fillDefault.value = query.c;
});

// debouncing since the fzf and svg is actually a bit of a grind for every key press
watch(
  mapData,
  _.debounce(async () => {
    // if we filtered away our selection remove it
    if (
      selectedComponent.value?.id &&
      !mapData.value.components[selectedComponent.value.id]
    )
      selectedComponent.value = null;

    const children: node[] = [...mapData.value.nodes].map((nId) => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const component = mapData.value.components[nId]!;
      const icons: [string | null] = [
        component.hasResource ? "check-hex-success" : null,
      ];
      if (component.qualificationTotals.failed > 0)
        icons.push("x-hex-outline-destructive");
      else icons.push("check-hex-outline-success");
      return { id: nId, width: WIDTH, height: HEIGHT, component, icons };
    });
    const edges: edge[] = [...mapData.value.edges].map((eId) => {
      const [target, source] = eId.split("-");
      return { id: eId, sources: [source ?? ""], targets: [target ?? ""] };
    });
    if (!children || !edges) return null;
    const graph = {
      id: "root",
      layoutOptions: {
        "elk.algorithm": "layered",
        direction: "DOWN",
        spacing: "80",
        "layered.spacing.nodeNodeBetweenLayers": "150",
        "spacing.nodeNode": "80",
        padding: "50",
      },
      children,
      edges,
    };
    const elk = new ELK();
    const layoutedData = await elk.layout(graph);
    dataAsGraph.value = layoutedData;
    nextTick(() => {
      const svg = d3.select("#map > svg g");
      // the viewbox should be based on "how much of the coordinate space is in use"

      // the types generated aren't exactly matching the actual data!
      // this is what I see
      const children = layoutedData.children as layoutNode[];
      const edges = layoutedData.edges as layoutLine[];

      // clear out for a redraw
      svg.selectAll("*").remove();

      const groups = svg
        .selectAll(".node")
        .data(children)
        .enter()
        .append("g")
        .attr("transform", (d) => `translate(${d.x}, ${d.y})`);

      groups
        .append("rect")
        .attr("width", (d) => d.width)
        .attr("height", (d) => d.height)
        // note this only handles the selected class on load
        .attr(
          "class",
          (d) =>
            `node id-${d.component.id} ${
              selectedComponent.value?.id === d.component.id && "selected"
            }`,
        )
        .on("click", (e: MouseEvent, d: layoutNode) => {
          clickedNode(e, d);
        })
        .on("contextmenu", (e: MouseEvent, d: layoutNode) => {
          clickedNode(e, d);
        })
        .on("dblclick", (_e: Event, d: layoutNode) => {
          componentNavigate(d.component.id);
        });

      groups
        .append("path")
        .attr("d", () => {
          const lineGenerator = d3.line();
          return lineGenerator([
            [0, 0],
            [0, HEIGHT],
          ]);
        })
        .attr("stroke", (d) => d.component.color ?? "#111111")
        .attr("stroke-width", 3)
        .attr("pointer-events", "none"); // prevents this from being clickable

      // logos
      groups
        .append("path")
        .attr("d", d3.symbol().size(1000).type(d3.symbolSquare))
        .attr("transform", () => {
          return "translate(23, 35)";
        })
        .attr("pointer-events", "none") // prevents this from being clickable
        .style("fill", (d) => {
          const icon = getAssetIcon(d.component.schemaCategory);
          return `url(#${icon})`;
        });

      // TODO ellipsis / truncation
      groups
        .append("text")
        .text((d) => d.component.name)
        .attr("dx", "45")
        .attr("dy", "25")
        .attr("class", "name")
        .attr("alignment-baseline", "middle")
        .attr("pointer-events", "none"); // prevents this from being clickable

      groups
        .append("text")
        .text((d) => d.component.schemaVariantName)
        .attr("dx", "45")
        .attr("dy", "45")
        .attr("alignment-baseline", "middle")
        .attr("color", "white")
        .attr("pointer-events", "none"); // prevents this from being clickable

      // qual & resource icons
      groups.each(function doTheIcons(d) {
        d.icons.forEach((icon, idx) => {
          d3.select(this)
            .append("path")
            // i have zero idea what this size param does
            // smaller value the SVG gets bigger and blurry
            // 1000 value its "correct" & sharp
            // even larger and it starts moving on
            .attr("d", d3.symbol().size(1000).type(d3.symbolSquare))
            .attr("transform", () => {
              return `translate(${WIDTH - 13 - 23 * idx}, ${HEIGHT - 9})`;
            })
            .attr("pointer-events", "none")
            .style("fill", () => {
              return icon ? `url(#${icon})` : "none";
            });
        });
      });

      svg
        .selectAll(".edge")
        .data(edges)
        .enter()
        .append("path")
        .attr("class", "edge")
        .attr("d", (d) => {
          const lineGenerator = d3.line();

          const points: xy[] = [];
          d.sections.forEach((section) => {
            points.push({ x: section.startPoint.x, y: section.startPoint.y });
            if (section.bendPoints) points.push(...section.bendPoints);
            points.push({ x: section.endPoint.x, y: section.endPoint.y });
          });
          const pairs: Array<[number, number]> = points.map((p) => [p.x, p.y]);
          return lineGenerator(pairs);
        })
        .style("fill", "none");
    });
  }, 100),
  { immediate: true },
);

function getButtonClasses(isDisabled: boolean) {
  return clsx(
    tw`rounded-full p-1 border`,
    themeClasses(
      "bg-neutral-600 text-white active:bg-neutral-200 hover:bg-neutral-400 active:text-black active:border-black",
      "bg-neutral-200 text-black active:bg-neutral-700 hover:bg-neutral-400 active:text-white active:border-white",
    ),
    isDisabled ? tw`cursor-not-allowed opacity-50` : tw`cursor-pointer`,
  );
}

const route = useRoute();
const componentNavigate = (componentId: ComponentId) => {
  const params = { ...route.params };
  params.componentId = componentId;
  router.push({
    name: "new-hotness-component",
    params,
    query: {},
  });
};
</script>

<style lang="less">
#map > svg {
  rect.node {
    fill: @colors-neutral-800;
    stroke: @colors-neutral-500;
    stroke-width: 2;

    &.selected {
      fill: @colors-action-800;
    }

    &:hover {
      fill: @colors-neutral-700;
      cursor: pointer;
    }

    &.selected:hover {
      fill: @colors-action-700;
    }
  }

  text {
    fill: white;
    font-size: 1rem;
    &.name {
      font-weight: bold;
      font-size: 1.2rem;
    }
  }

  path.edge {
    stroke: @colors-neutral-500;
  }
}
</style>
