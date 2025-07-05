<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <section id="map" class="grid h-full">
    <div
      v-if="selectedComponent"
      id="selection"
      :class="
        clsx(
          'absolute top-[110px] max-w-[350px] min-w-[300px] right-3',
          'flex flex-col gap-xs',
        )
      "
      style="max-height: calc(100vh - 115px)"
    >
      <div
        :class="
          clsx(
            'flex-none p-xs border-2',
            themeClasses(
              'border-action-300 bg-action-200',
              'border-action-700 bg-action-800',
            ),
          )
        "
      >
        <ExploreGridTile
          ref="selectedGridTileRef"
          :component="selectedComponent"
          hideConnections
          @click="navigateToSelectedComponent"
        />
      </div>
      <div class="scrollable grow">
        <ConnectionsPanel :component="selectedComponent" noEmptyState />
      </div>
    </div>

    <div
      id="controls"
      class="absolute left-0 bottom-0 flex flex-row gap-xs m-sm items-center"
    >
      <div
        v-tooltip="'Zoom Out'"
        :class="getButtonClasses(zoomLevel >= MAX_ZOOM)"
        @click="zoomOut"
      >
        <Icon name="minus" size="sm" />
      </div>
      <div
        v-tooltip="'Current Zoom'"
        :class="
          clsx(
            'border p-2xs rounded select-none',
            themeClasses(
              'text-black border-black bg-white',
              'text-white border-white bg-black',
            ),
          )
        "
      >
        {{ Math.round(zoomLevel * 100) }}%
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
      <div
        v-tooltip="'Help'"
        :class="getButtonClasses(false)"
        @click="emit('help')"
      >
        <Icon name="question-circle" size="sm" />
      </div>
    </div>

    <svg
      :class="mouseDown ? 'cursor-grabbing' : 'cursor-grab'"
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
          <IconNoWrapper
            :name="logo"
            :class="themeClasses('text-black', 'text-white')"
          />
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
      onGrid
      @edit="navigateToSelectedComponent"
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
  onMounted,
  reactive,
  ref,
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
  IncomingConnections,
  ComponentInList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import {
  bifrostList,
  useMakeArgs,
  useMakeKey,
} from "@/store/realtime/heimdall";
import { SelectionsInQueryString } from "./Workspace.vue";
import { KeyDetails } from "./logic_composables/emitters";
import { assertIsDefined, Context, ExploreContext } from "./types";
import ExploreGridTile from "./explore_grid/ExploreGridTile.vue";
import ConnectionsPanel from "./ConnectionsPanel.vue";
import { getAssetIcon } from "./util";
import ComponentContextMenu from "./ComponentContextMenu.vue";
import { truncateString } from "./logic_composables/string_funcs";

const MAX_STRING_LENGTH = 18;

const props = defineProps<{
  active: boolean;
  components: ComponentInList[];
}>();

const componentsById = computed<Record<ComponentId, ComponentInList>>(() => {
  return props.components.reduce((obj, component) => {
    obj[component.id] = component;
    return obj;
  }, {} as Record<ComponentId, ComponentInList>);
});

const componentContextMenuRef =
  ref<InstanceType<typeof ComponentContextMenu>>();

const selectedComponent = ref<ComponentInList | null>(null);

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

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

  // TODO(Wendy) - this fixes the position of the context menu
  // but it is not a perfect fix, as it doesn't handle the
  // selected component going near the edges or offscreen well :(
  if (selectedComponent.value) selectComponent(selectedComponent.value);
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

const clickWithNoDrag = ref(false);

const mousedown = () => {
  mouseDown.value = true;
  clickWithNoDrag.value = true;
};

const mouseup = (e: MouseEvent) => {
  mouseDown.value = false;
  const target = e.target;
  if (target instanceof SVGRectElement && target.classList.contains("node")) {
    // don't deselect if you clicked on a node!
    return;
  } else if (clickWithNoDrag.value) {
    deselect();
    emit("deselect");
  }
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
  deselect();
};
const onE = (_e: KeyDetails["e"]) => {
  if (selectedComponent.value) {
    componentContextMenuRef.value?.componentsStartErase([
      selectedComponent.value,
    ]);
  }
};
const onD = (e: KeyDetails["d"]) => {
  if (selectedComponent.value && (e.metaKey || e.ctrlKey)) {
    componentContextMenuRef.value?.componentsDuplicate([
      selectedComponent.value.id,
    ]);
  }
};
const onP = (_e: KeyDetails["p"]) => {
  // Do nothing! Pinning is unsupported in the map view.
};
const onU = (_e: KeyDetails["u"]) => {
  // if (selectedComponent.value && selectedComponent.value.canBeUpgraded) {
  //   componentContextMenuRef.value?.componentUpgrade([
  //     selectedComponent.value.id,
  //   ]);
  // }
};
const onBackspace = (_e: KeyDetails["Backspace"]) => {
  if (selectedComponent.value && !selectedComponent.value.toDelete) {
    componentContextMenuRef.value?.componentsStartDelete([
      selectedComponent.value,
    ]);
  }
};
const onR = (_e: KeyDetails["r"]) => {
  if (selectedComponent.value && selectedComponent.value.toDelete) {
    componentContextMenuRef.value?.componentsRestore([
      selectedComponent.value.id,
    ]);
  }
};
onMounted(() => {
  // if we need to adjust zoom level on load dynamically
  // change it here
  applyZoom();
});

const mousemove = (event: MouseEvent) => {
  clickWithNoDrag.value = false;
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

const connections = useQuery<IncomingConnections[]>({
  queryKey,
  enabled: () => active.value, // Only run query when map view is active
  queryFn: async () => {
    const d = await bifrostList<IncomingConnections[] | null>(
      args(EntityKind.IncomingConnectionsList),
    );

    if (d) {
      // this sets the component from the URL querystring on load, and then doesn't re-enter
      if (fillDefault.value) {
        nextTick(() => {
          const c = d.find((c) => c.id === fillDefault.value);
          if (c && c.id) {
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            selectComponent(componentsById.value[c.id]!);
          }
          fillDefault.value = undefined;
        });
      }

      return d;
    } else return [];
  },
});

const mapData = computed(() => {
  const nodes = new Set<string>();
  const edges = new Set<string>();
  const components: Record<string, ComponentInList> = {};
  if (!connections.data.value) {
    return { nodes, edges, components };
  }

  const matchingIds: string[] = [];
  if (searchString?.value && searchString.value.trim().length > 0) {
    const searchTerm = searchString.value.trim();

    // Check if the search term contains "schema:" queries
    const schemaMatches = searchTerm.match(/schema:([^\s]+)/gi);
    if (schemaMatches) {
      const schemaNames = schemaMatches.map((match) =>
        match.substring(7).trim().toLowerCase(),
      );

      if (schemaNames.length === 0 || schemaNames.some((name) => name === "")) {
        matchingIds.push(...Object.keys(componentsById.value));
      } else {
        const results = Object.values(componentsById.value).filter((c) =>
          schemaNames.includes(c.schemaName.toLowerCase()),
        );
        matchingIds.push(...results.map((c) => c.id));
      }
    } else {
      // Regular fuzzy search across all fields
      const fzf = new Fzf(Object.values(componentsById.value), {
        casing: "case-insensitive",
        selector: (c) =>
          `${c.name} ${c.schemaVariantName} ${c.schemaName} ${c.schemaCategory} ${c.schemaId} ${c.id}`,
      });

      const results = fzf.find(searchTerm);
      if (results.length === 0) return { nodes, edges, components };
      else matchingIds.push(...results.map((c) => c.item.id));
    }
  } else {
    matchingIds.push(...Object.keys(componentsById.value));
  }

  connections.data.value.forEach((c) => {
    if (!matchingIds.includes(c.id)) return;

    nodes.add(c.id);
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    components[c.id] = componentsById.value[c.id]!;
    c.connections.forEach((e) => {
      // incoming, so "to" is me, always start with "me"
      if (
        !matchingIds.includes(e.toComponentId) ||
        !matchingIds.includes(e.fromComponentId)
      )
        return;

      const edge = `${e.toComponentId}-${e.fromComponentId}`;
      edges.add(edge);
    });
  });

  return { nodes, edges, components };
});

type node = {
  id: string;
  width: number;
  height: number;
  component: ComponentInList;
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

const selectedGridTileRef = ref<InstanceType<typeof ExploreGridTile>>();

const router = useRouter();
const clickedNode = (e: MouseEvent, n: layoutNode) => {
  e.preventDefault();
  e.stopPropagation();

  if (selectedComponent.value?.id === n.component.id) {
    navigateToSelectedComponent();
  } else {
    selectComponent(n.component, e.target as Element);
  }
};

const selectComponent = (component: ComponentInList, componentEl?: Element) => {
  selectedComponent.value = component;

  nextTick(() => {
    let element = componentEl;
    if (!element) {
      element = document.getElementsByClassName(`id-${component.id}`)[0];
      if (!element) return;
    }

    const rect = element.getBoundingClientRect();

    const anchor = {
      $el: element,
      getBoundingClientRect: () => ({
        ...rect,
        left: rect.right,
        x: rect.right,
        width: 0,
      }),
    };

    componentContextMenuRef.value?.open(anchor, [component]);
  });
};
const deselect = () => {
  selectedComponent.value = null;
  componentContextMenuRef.value?.close();
};

watch(selectedComponent, () => {
  // this handles later changes after the page loads
  document.querySelectorAll("#map > svg rect.node").forEach((n) => {
    n.classList.remove("selected");
    n.classList.remove("greyed-out");
  });

  // Remove greyed-out classes from text elements
  document
    .querySelectorAll("#map > svg text")
    .forEach((n) => n.classList.remove("greyed-out"));

  // Reset opacity for all icons and color bars
  document.querySelectorAll("#map > svg path").forEach((path) => {
    if (path.getAttribute("stroke")) {
      path.setAttribute("stroke-opacity", "1");
    } else {
      (path as HTMLElement).style.opacity = "1";
    }
  });

  const query: SelectionsInQueryString = {
    ...router.currentRoute.value?.query,
  };
  delete query.c;

  if (selectedComponent.value) {
    query.c = selectedComponent.value.id;

    // Add selected class to the selected component
    document
      .querySelector(`#map > svg rect.node.id-${selectedComponent.value.id}`)
      ?.classList.add("selected");

    // Add greyed-out class to unconnected components
    document.querySelectorAll("#map > svg rect.node").forEach((element) => {
      const componentId = Array.from(element.classList)
        .find((cls) => cls.startsWith("id-"))
        ?.substring(3);

      if (componentId && !connectedComponentIds.value.has(componentId)) {
        element.classList.add("greyed-out");
      }
    });

    // Add greyed-out class to text elements of unconnected components
    document.querySelectorAll("#map > svg g").forEach((group) => {
      const rect = group.querySelector("rect.node");
      const componentId = Array.from(rect?.classList || [])
        .find((cls) => cls.startsWith("id-"))
        ?.substring(3);

      if (componentId && !connectedComponentIds.value.has(componentId)) {
        group.querySelectorAll("text").forEach((text) => {
          text.classList.add("greyed-out");
        });

        // Grey out icons and color bars
        group.querySelectorAll("path").forEach((path) => {
          if (path.getAttribute("stroke")) {
            // This is likely a color bar
            path.setAttribute("stroke-opacity", "0.3");
          } else {
            // This is likely an icon
            path.style.opacity = "0.3";
          }
        });
      }
    });
  }

  router.push({ query });
});

const searchString = inject<ComputedRef<string>>("SEARCH");

const connectedComponentIds = computed(() => {
  const connectedIds = new Set<string>();
  if (!selectedComponent.value || !connections.data.value) {
    return connectedIds;
  }

  const selectedId = selectedComponent.value.id;
  connectedIds.add(selectedId); // Include the selected component itself

  // Find all components connected to the selected component
  connections.data.value.forEach((component) => {
    component.connections.forEach((connection) => {
      if (connection.toComponentId === selectedId) {
        connectedIds.add(connection.fromComponentId);
      }
      if (connection.fromComponentId === selectedId) {
        connectedIds.add(connection.toComponentId);
      }
    });
  });

  return connectedIds;
});

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
      deselect();

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

      groups
        .append("text")
        .text((d) => truncateString(d.component.name, MAX_STRING_LENGTH))
        .attr("dx", "45")
        .attr("dy", "25")
        .attr("class", "name")
        .attr("alignment-baseline", "middle")
        .attr("pointer-events", "none"); // prevents this from being clickable

      groups
        .append("text")
        .text((d) =>
          truncateString(d.component.schemaVariantName, MAX_STRING_LENGTH),
        )
        .attr("dx", "45")
        .attr("dy", "45")
        .attr("class", "")
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
    if (selectedComponent.value && !componentContextMenuRef.value?.isOpen) {
      selectComponent(selectedComponent.value);
    }
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

const navigateToSelectedComponent = () => {
  if (selectedComponent.value?.id) {
    componentNavigate(selectedComponent.value.id);
  }
};

const emit = defineEmits<{
  (e: "deselect"): void;
  (e: "help"): void;
}>();

defineExpose({
  deselect,
  navigateToSelectedComponent,
  onArrowUp,
  onArrowDown,
  onArrowLeft,
  onArrowRight,
  onEscape,
  onE,
  onD,
  onP,
  onU,
  onBackspace,
  onR,
});
</script>

<style lang="less">
#map > svg {
  rect.node {
    fill: @colors-neutral-100;
    stroke: @colors-neutral-500;
    stroke-width: 2;

    body.dark & {
      fill: @colors-neutral-800;
    }

    &.selected {
      fill: @colors-action-200;
      stroke: @colors-action-500;
      body.dark & {
        fill: @colors-action-800;
        stroke: @colors-action-300;
      }
    }

    &:hover {
      fill: @colors-neutral-200;
      body.dark & {
        fill: @colors-neutral-700;
      }
      cursor: pointer;
    }

    &.selected:hover {
      fill: @colors-action-300;
      body.dark & {
        fill: @colors-action-300;
      }
    }

    &.greyed-out {
      fill: @colors-neutral-200;
      stroke: @colors-neutral-300;
      opacity: 0.3;

      body.dark & {
        fill: @colors-neutral-700;
        stroke: @colors-neutral-600;
      }
    }
  }

  text {
    fill: black;
    body.dark & {
      fill: white;
    }
    font-size: 1rem;
    &.name {
      font-weight: bold;
      font-size: 1.2rem;
    }

    &.greyed-out {
      fill: @colors-neutral-400;
      opacity: 0.6;

      body.dark & {
        fill: @colors-neutral-500;
      }
    }
  }

  path.edge {
    stroke: @colors-neutral-500;
  }
}
</style>
