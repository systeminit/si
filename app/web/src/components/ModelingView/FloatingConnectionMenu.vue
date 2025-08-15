<template>
  <Modal
    ref="modalRef"
    :capitalizeTitle="false"
    hideExitButton
    size="max"
    title="Create a connection"
    @click="focusOnInput"
    @close="onClose"
  >
    <template #titleIcons>
      <div class="text-xs cursor-pointer" @click="modalRef?.close()">
        <TextPill variant="key">ESC</TextPill> to exit
      </div>
    </template>
    <div
      class="flex flex-col h-[80vh] rounded-sm children:gap-sm"
      @click="focusOnInput"
      @keydown="debouncedListener"
    >
      <div
        v-if="featureFlagsStore.PROPS_TO_PROPS_CONNECTIONS"
        class="flex justify-between w-full my-sm"
      >
        <div class="grow basis-0">
          {{ sideATitle }}
        </div>
        <div class="">
          <IconButton
            v-if="connectionData.aDirection"
            :class="
              clsx([
                'transition-transform',
                connectionData.aDirection === 'input' ? '-rotate-180' : '',
              ])
            "
            icon="arrow--right"
            iconTone="neutral"
            @click="invertModalDirection"
          />
        </div>
        <div class="grow basis-0 text-end">
          {{ sideBTitle }}
        </div>
      </div>
      <div class="flex flex-row w-full children:basis-1/2">
        <FloatingConnectionMenuInput
          ref="inputARef"
          :active="activeSide === 'a'"
          @click="undoASelection"
        />
        <FloatingConnectionMenuInput
          ref="inputBRef"
          :active="activeSide === 'b'"
          :disabled="activeSide === 'a'"
        />
      </div>
      <div class="flex flex-row grow min-h-0 children:basis-1/2">
        <!-- Socket A -->
        <ConnectionMenuCandidateList
          :active="activeSide === 'a'"
          :controlScheme="controlScheme"
          :doneLoading="doneOpening"
          :filteringBySearchString="searchStringA"
          :highlightedIndex="highlightedIndex"
          :highlightedSocket="highlightedListEntry"
          :listItems="fullASideList"
          :selectedComponent="selectedComponentA"
          :selectedSocket="selectedSocketA"
          @select="(index: number) => selectAndProcess('a', index)"
        />
        <!-- Socket B -->
        <ConnectionMenuCandidateList
          :active="activeSide === 'b'"
          :controlScheme="controlScheme"
          :doneLoading="doneOpening"
          :filteringBySearchString="searchStringB"
          :highlightedIndex="highlightedIndex"
          :highlightedSocket="highlightedListEntry"
          :listItems="fullBSideList"
          :selectedComponent="selectedComponentB"
          :selectedSocket="selectedSocketB"
          @select="(index: number) => selectAndProcess('b', index)"
        />
      </div>
    </div>
    <div
      class="flex w-full h-8 mt-sm items-center justify-between gap-sm text-xs"
    >
      <div>
        <span
          v-if="fetchDataForAllViewsStatus.isPending"
          class="flex items-center gap-xs"
        >
          <Icon name="loader" size="sm" /> Loading data for other views. Some
          connection candidates may not be shown yet.
        </span>
        <span v-else-if="runningUpdateMenu" class="flex items-center gap-xs">
          <Icon name="loader" size="sm" /> Computing possible connections...
        </span>
      </div>
      <div class="flex gap-sm">
        <div class="flex gap-2xs items-center">
          <TextPill variant="key">Up</TextPill>
          <TextPill variant="key">Down</TextPill>
          <div>to navigate</div>
        </div>
        <div
          :class="
            clsx(
              'border-l h-full',
              themeClasses('border-neutral-300', 'border-neutral-600'),
            )
          "
        />
        <div class="flex flex-row gap-2xs items-center">
          <TextPill variant="key">
            {{ controlScheme === "arrows" ? "Right" : "Tab" }}
          </TextPill>
          <template v-if="controlScheme === 'arrows'">
            <div>or</div>
            <TextPill variant="key">Enter</TextPill>
          </template>
          <div>to select</div>
        </div>
        <div
          :class="
            clsx(
              'border-l h-full',
              themeClasses('border-neutral-300', 'border-neutral-600'),
            )
          "
        />
        <div class="flex flex-row gap-2xs items-center">
          <TextPill variant="key">
            {{ controlScheme === "arrows" ? "Left" : "Shift + Tab" }}
          </TextPill>
          <div>to switch back to the list on the left</div>
        </div>
        <template v-if="controlScheme !== 'arrows'">
          <div
            :class="
              clsx(
                'border-l h-full',
                themeClasses('border-neutral-300', 'border-neutral-600'),
              )
            "
          />
          <div class="flex flex-row gap-2xs items-center">
            <TextPill variant="key">Enter</TextPill>
            <div>to create a connection</div>
          </div>
        </template>
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  Modal,
  themeClasses,
  Icon,
  IconButton,
  TextPill,
} from "@si/vue-lib/design-system";
import {
  computed,
  nextTick,
  onMounted,
  onUnmounted,
  reactive,
  ref,
  watch,
} from "vue";
import clsx from "clsx";
import { Fzf } from "fzf";
import {
  ConnectionDirection,
  ConnectionMenuData,
  generateSocketPaths,
  SocketWithParent,
  useComponentsStore,
} from "@/store/components.store";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramSocketData,
  DiagramSocketEdgeData,
} from "@/components/ModelingDiagram/diagram_types";
import { useViewsStore } from "@/store/views.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import { PropertyEditorSchema } from "@/api/sdf/dal/property_editor";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import FloatingConnectionMenuInput from "./FloatingConnectionMenuInput.vue";
import ConnectionMenuCandidateList, {
  ConnectionCandidateSocket,
  ConnectionCandidateListEntry,
  ConnectionCandidateProp,
  candidateIsSocket,
  candidateIsProp,
} from "./ConnectionMenuCandidateList.vue";

const modalRef = ref<InstanceType<typeof Modal>>();
const inputARef = ref<InstanceType<typeof FloatingConnectionMenuInput>>();
const inputBRef = ref<InstanceType<typeof FloatingConnectionMenuInput>>();
const searchStringA = computed(() => inputARef.value?.searchString);
const searchStringB = computed(() => inputBRef.value?.searchString);
const startingSearchStringA = ref("");
const startingSearchStringB = ref("");

const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();
const featureFlagsStore = useFeatureFlagsStore();

const fetchDataForAllViewsStatus = componentsStore.getRequestStatus(
  "FETCH_ALL_COMPONENTS",
);

const doneOpening = ref(false);

const controlScheme = "arrows";

onMounted(() => {
  componentsStore.eventBus.on("openConnectionsMenu", open);
});

onUnmounted(() => {
  componentsStore.eventBus.off("openConnectionsMenu", open);
});

const activeInputRef = computed(() =>
  activeSide.value === "a" ? inputARef : inputBRef,
);

const listAItems = ref<ConnectionCandidateListEntry[]>([]);
const listBItems = ref<ConnectionCandidateListEntry[]>([]);

const focusOnInput = () => {
  nextTick(() => {
    activeInputRef.value.value?.focus();
  });
};

const fixedDirection = ref<ConnectionDirection | undefined>(undefined);

// Selection data
const connectionData = reactive<ConnectionMenuData>({
  aDirection: undefined,
  A: {
    componentId: undefined,
    socketId: undefined,
    attributePath: undefined,
  },
  B: {
    componentId: undefined,
    socketId: undefined,
    attributePath: undefined,
  },
});

const invertModalDirection = () => {
  connectionData.aDirection = invertDirection(connectionData.aDirection);
};

const invertDirection = (direction?: string) => {
  switch (direction) {
    case "input":
      return "output";
    case "output":
      return "input";
    default:
      return undefined;
  }
};

const selectedComponentA = computed(
  () => componentsStore.allComponentsById[connectionData.A.componentId ?? ""],
);
const selectedComponentB = computed(
  () => componentsStore.allComponentsById[connectionData.B.componentId ?? ""],
);
const selectedSocketA = computed(() =>
  selectedComponentA.value?.sockets.find(
    (s) => s.def.id === connectionData.A.socketId,
  ),
);
const selectedSocketB = computed(() =>
  selectedComponentB.value?.sockets.find(
    (s) => s.def.id === connectionData.B.socketId,
  ),
);

const sideATitle = computed(() => {
  const direction =
    connectionData.aDirection === "input" ? "Destination" : "Source";

  return `${direction} value`;
});

const sideBTitle = computed(() => {
  const direction =
    connectionData.aDirection === "input" ? "Source" : "Destination";

  return `${direction} value`;
});

// this needs to compute based on allComponentsById changes to socket values
const highlightedListEntry = computed(() => {
  const highlighted =
    activeSide.value === "a"
      ? listAItems.value[highlightedIndex.value]
      : listBItems.value[highlightedIndex.value];
  if (!highlighted) return undefined;

  if (candidateIsSocket(highlighted)) {
    return componentsStore.allComponentsById[
      highlighted.component.def.id
    ]?.sockets.find((s) => s.def.id === highlighted.socket?.def.id);
  }

  // TODO highlighted prop
  return undefined;
});

const activeSide = ref<"a" | "b">("a");
// TODO this will undo the previous connection
const undoASelection = () => {
  if (activeSide.value === "a") {
    return;
  }
  activeSide.value = "a";
  connectionData.A = {};
  connectionData.B = {};
  focusOnInput();
};

const runningUpdateMenu = ref(false);

const searchStrings = computed(() => {
  const [active, inactive] =
    activeSide.value === "a"
      ? [searchStringA.value, searchStringB.value]
      : [searchStringB.value, searchStringA.value];

  return {
    active,
    inactive,
  };
});

// Generate the options on both sides
// TODO Break this into computed variables
const updateMenu = async () => {
  if (!modalRef.value?.isOpen) {
    return;
  }
  runningUpdateMenu.value = true;

  const [activeSideData, otherSideData] =
    activeSide.value === "a"
      ? [connectionData.A, connectionData.B]
      : [connectionData.B, connectionData.A];
  const [activeSideList, otherSideList] =
    activeSide.value === "a"
      ? [listAItems, listBItems]
      : [listBItems, listAItems];

  const edges = _.values(componentsStore.diagramEdgesById).filter(
    (e) => e.def.toDelete === false && e.def.changeStatus !== "deleted",
  );

  const componentsActiveSide = _.filter(
    activeSideData.componentId
      ? _.compact([
          componentsStore.allComponentsById[activeSideData.componentId],
        ])
      : _.values(componentsStore.allComponentsById),
    // Do not get the peer component, if one is selected
    (c) => !otherSideData.componentId || c.def.id !== otherSideData.componentId,
  );

  // Gather all sockets on the active side
  const activeSideSockets = _.flatten(
    componentsActiveSide?.map((c) =>
      c.sockets
        .filter((s) => !s.def.isManagement)
        .filter((s) => {
          // If the panel is opened with a fixed direction, restrict which sockets are allowed based on it while the search string includes the original path
          if (
            fixedDirection.value &&
            ((activeSide.value === "a" &&
              s.def.direction !== fixedDirection.value) ||
              (activeSide.value === "b" &&
                s.def.direction === fixedDirection.value)) &&
            searchStringA.value?.includes(startingSearchStringA.value)
          ) {
            return false;
          }

          // Output sockets or input sockets that do not have a max number of connections always make it through this filter
          if (s.def.direction !== "input" || s.def.maxConnections === null) {
            return true;
          }

          // this is to check if the socket is full already
          const incomingConnections = edges.filter(
            (e) =>
              e instanceof DiagramSocketEdgeData &&
              e.toSocketKey === s.uniqueKey,
          );

          return incomingConnections.length < s.def.maxConnections;
        })
        .map((s) => ({
          component: c,
          socket: s,
        }))
        .reduce((acc, entry) => {
          const paths = generateSocketPaths(entry.socket, viewsStore);
          for (const path of paths) {
            acc.push({ ...entry, label: path });
          }

          return acc;
        }, [] as ConnectionCandidateSocket[]),
    ),
  );

  const sortFn = (
    s1: ConnectionCandidateSocket,
    s2: ConnectionCandidateSocket,
  ) => {
    if (s1.socket.def.direction === s2.socket.def.direction) {
      if (s1.label < s2.label) {
        return -1;
      }
      if (s1.label > s2.label) {
        return 1;
      }
      return 0;
    }
    if (s1.socket?.def.direction === "input") {
      return -1;
    } else {
      return 1;
    }
  };

  activeSideSockets.sort(sortFn);

  let matchedActiveSideSockets = activeSideSockets;
  if (searchStrings.value.active) {
    const fzf = new Fzf(activeSideSockets, {
      casing: "case-insensitive",
      selector: (item) => item.label,
    });

    matchedActiveSideSockets = fzf
      .find(searchStrings.value.active)
      .map((e) => ({
        ...e.item,
        labelHighlights: e.positions,
      }));
  }

  // Get all valid peer sockets for the active sockets
  const peersFunction = componentsStore.possibleAndExistingPeerSocketsFn;
  type SocketListEntryWithOriginator = ConnectionCandidateSocket & {
    originatorSocketPathKey: string;
  };

  // This goofy promise and timeout structure exists so we don't block the UI when running a thousand
  const allPossiblePeers = _.flatten(
    await Promise.all(
      matchedActiveSideSockets.map(
        ({ component, socket, label }) =>
          new Promise((resolve) => {
            setTimeout(() => {
              resolve(
                peersFunction(socket.def, component.def.id).possiblePeers.map(
                  (p) => ({
                    originatorSocketPathKey: generateUniquePathKey(
                      label,
                      socket,
                    ),
                    ...p,
                  }),
                ),
              );
            }, 0);
          }),
      ),
    ),
  ) as Array<
    SocketWithParent & {
      originatorSocketPathKey: string;
    }
  >;

  const validPeers = allPossiblePeers
    .filter(
      (s) =>
        !otherSideData.componentId ||
        otherSideData.componentId === s.componentId,
    )
    .filter((s) => !otherSideData.socketId || otherSideData.socketId === s.id)
    .map((s) => {
      const component = componentsStore.allComponentsById[s.componentId];

      if (!component) {
        return;
      }

      return {
        originatorSocketPathKey: s.originatorSocketPathKey,
        component,
        socket: new DiagramSocketData(component, s),
      };
    })
    .reduce((acc, entry) => {
      if (!entry) return acc;

      const paths = generateSocketPaths(entry.socket, viewsStore);
      for (const path of paths) {
        acc.push({ ...entry, label: path });
      }

      return acc;
    }, [] as SocketListEntryWithOriginator[]);

  // Remove the active side sockets without peers
  const otherSideSockets = {} as Record<string, ConnectionCandidateSocket>;
  const socketsWithPossiblePeers = new Set();
  for (const peer of _.compact(validPeers)) {
    otherSideSockets[peer.label] = peer;
    socketsWithPossiblePeers.add(peer.originatorSocketPathKey);
  }

  const activeSideSocketsWithPeers = matchedActiveSideSockets.filter((e) =>
    socketsWithPossiblePeers.has(generateUniquePathKey(e.label, e.socket)),
  );

  const deduplicatedOtherSideSockets = _.values(otherSideSockets);

  deduplicatedOtherSideSockets.sort(sortFn);

  let matchedOtherSideSockets = deduplicatedOtherSideSockets;
  if (searchStrings.value.inactive) {
    const fzf = new Fzf(deduplicatedOtherSideSockets, {
      casing: "case-insensitive",
      selector: (item) => item.label,
    });

    matchedOtherSideSockets = fzf
      .find(searchStrings.value.inactive)
      .map((e) => ({
        ...e.item,
        labelHighlights: e.positions,
      }));
  }

  activeSideList.value = activeSideSocketsWithPeers;
  otherSideList.value = matchedOtherSideSockets;

  runningUpdateMenu.value = false;
};
const debouncedUpdate = _.debounce(updateMenu, 20, {
  trailing: true,
});
watch(activeSide, debouncedUpdate, { immediate: true });
watch(searchStringA, debouncedUpdate, { immediate: true });
watch(searchStringB, debouncedUpdate, { immediate: true });
watch(connectionData, debouncedUpdate, { immediate: true });
watch(componentsStore.diagramEdgesById, debouncedUpdate, { immediate: true });
watch(componentsStore.allComponentsById, debouncedUpdate, { immediate: true });

const socketlessConnectionCandidatesA = ref<ConnectionCandidateProp[]>([]);
const socketlessConnectionCandidatesB = ref<ConnectionCandidateProp[]>([]);

const fullASideList = computed(() => [
  ...listAItems.value,
  ...socketlessConnectionCandidatesA.value,
]);

const fullBSideList = computed(() => {
  if (connectionData.A.socketId) {
    return [...listBItems.value];
  } else if (connectionData.A.attributePath) {
    return [...socketlessConnectionCandidatesB.value];
  }

  return [...listBItems.value, ...socketlessConnectionCandidatesB.value];
});

// Prop candidates
const schemaVariantsById = reactive({} as Record<string, PropertyEditorSchema>);
const storedInitialComponent = ref();
const loadingPropConnections = ref(false);
const loadedPropConnections = ref(false);
watch(
  [storedInitialComponent, searchStrings, connectionData.aDirection],
  async () => {
    if (!featureFlagsStore.PROPS_TO_PROPS_CONNECTIONS) {
      return;
    }

    if (!storedInitialComponent.value) return;
    loadingPropConnections.value = true;
    loadedPropConnections.value = false;

    const allComponents = _.values(componentsStore.allComponentsById).filter(
      (c) => c.def.changeStatus !== "deleted",
    );

    // Gather the schemas for the props currently on the diagram
    const componentsBySvId = {} as Record<
      string,
      (DiagramNodeData | DiagramGroupData)[]
    >;
    for (const component of allComponents) {
      const svId = component.def.schemaVariantId;
      componentsBySvId[svId] ??= [];
      componentsBySvId[svId]?.push(component);

      if (!schemaVariantsById[component.def.schemaVariantId]) {
        const attributesStore = useComponentAttributesStore(
          component.def.componentId,
        );
        await attributesStore.FETCH_PROPERTY_EDITOR_SCHEMA();
        if (attributesStore.schema) {
          schemaVariantsById[component.def.schemaVariantId] =
            attributesStore.schema;
        }
      }
    }

    let activeCandidates = [] as ConnectionCandidateProp[];
    let peerCandidates = [] as ConnectionCandidateProp[];

    for (const schemaId in schemaVariantsById) {
      const schema = schemaVariantsById[schemaId];
      if (!schema) continue;

      const queue = (schema.childProps[schema.rootPropId] ?? []).map(
        (propId) => ({ propId, parentPath: "" }),
      );

      const svPropPaths = [];
      while (queue.length > 0) {
        const entry = queue.shift();
        if (!entry) continue;
        const propId = entry.propId;
        const parentPath = entry.parentPath;

        const prop = schema.props[propId];
        if (!prop) continue;

        const path = `${parentPath}/${prop.name}`;

        if (
          !path.startsWith("/domain") &&
          !path.startsWith("/resource_value") &&
          !["/si", "/si/name", "/si/resourceId", "/si/color"].includes(path)
        )
          continue;

        svPropPaths.push(path);

        for (const childPropId of schema.childProps[propId] ?? []) {
          queue.unshift({ propId: childPropId, parentPath: path });
        }
      }

      for (const component of componentsBySvId[schemaId] ?? []) {
        const isActiveComponent =
          storedInitialComponent.value.def.id === component.def.id;

        const componentViews =
          viewsStore.viewNamesByComponentId[component.def.id] ?? [];

        for (const view of componentViews) {
          for (const propPath of svPropPaths) {
            const label = `${view}/${component.def.schemaName}/${component.def.displayName}${propPath}`;

            const entry = {
              component,
              propPath,
              label,
            };

            if (isActiveComponent) {
              activeCandidates.push(entry);
            } else {
              peerCandidates.push(entry);
            }
          }
          // Don't show options for all views for active component
          if (isActiveComponent) break;
        }
      }
    }

    const inputFilterFn = (e: ConnectionCandidateProp) =>
      !e.propPath.startsWith("/resource_value") &&
      e.propPath !== "/si" &&
      e.propPath !== "/si/resourceId";

    // Remove resource value from inputs
    if (connectionData.aDirection === "input") {
      activeCandidates = activeCandidates.filter(inputFilterFn);
    } else {
      peerCandidates = peerCandidates.filter(inputFilterFn);
    }

    if (searchStringA.value) {
      const fzf = new Fzf(activeCandidates, {
        casing: "case-insensitive",
        selector: (item) => item.label,
      });

      activeCandidates = fzf.find(searchStringA.value).map((e) => ({
        ...e.item,
        labelHighlights: e.positions,
      }));
    }

    if (searchStringB.value) {
      const fzf = new Fzf(peerCandidates, {
        casing: "case-insensitive",
        selector: (item) => item.label,
      });

      peerCandidates = fzf.find(searchStringB.value).map((e) => ({
        ...e.item,
        labelHighlights: e.positions,
      }));
    }

    socketlessConnectionCandidatesA.value = activeCandidates;
    socketlessConnectionCandidatesB.value = peerCandidates;
    loadingPropConnections.value = false;
    loadedPropConnections.value = true;
  },
);

const highlightedIndex = ref(0);
// Try to keep the selected option selected when list changes, else reset selected to 0
// watch(filteredListItems, (list, oldList) => {
//   const oldItem = oldList[selectedIndex.value];
//
//   const newId = list.findIndex((i) => i.def.id === oldItem?.def.id);
//
//   if (oldItem && newId >= 0) {
//     selectedIndex.value = newId;
//     return;
//   }
//
//   selectedIndex.value = 0;
// })

const keyListener = (e: KeyboardEvent) => {
  if (controlScheme === "arrows") {
    switch (e.key) {
      case "ArrowUp": {
        highlightPrev();
        e.preventDefault();
        break;
      }
      case "ArrowDown": {
        highlightNext();
        e.preventDefault();
        break;
      }
      case "ArrowRight": {
        processHighlighted();
        e.preventDefault();
        break;
      }
      case "Enter": {
        processHighlighted();
        e.preventDefault();
        break;
      }
      case "ArrowLeft": {
        undoASelection();
        e.preventDefault();
        break;
      }
      default:
        break;
    }
  } else {
    switch (e.key) {
      case "ArrowUp": {
        highlightPrev();
        e.preventDefault();
        break;
      }
      case "ArrowDown": {
        highlightNext();
        e.preventDefault();
        break;
      }
      case "Enter": {
        processHighlighted();
        e.preventDefault();
        break;
      }
      case "Tab": {
        if (e.shiftKey) undoASelection();
        else processHighlighted();
        e.preventDefault();
        break;
      }
      default:
        break;
    }
  }
};
const debouncedListener = _.debounce(keyListener, 10, {
  leading: true,
  trailing: false,
});

const highlightNext = () => {
  const upperLimit =
    (activeSide.value === "a"
      ? fullASideList.value.length
      : fullBSideList.value.length) - 1;

  highlightedIndex.value = Math.min(highlightedIndex.value + 1, upperLimit);
};
const highlightPrev = () => {
  highlightedIndex.value = Math.max(highlightedIndex.value - 1, 0);
};

const selectAndProcess = (side: "a" | "b", index: number) => {
  highlightedIndex.value = index;
  activeSide.value = side;
  processHighlighted();
};

watch(activeSide, () => {
  highlightedIndex.value = 0;
});

// NOTE(victor): This code was written with the expectation that you could go from B to A without unsetting data on A.
// this has since changed, but some overengineering was kept in case we change our minds
const processHighlighted = () => {
  if (activeSide.value === "a") {
    const selectedItem = fullASideList.value[highlightedIndex.value];
    if (!selectedItem) return;

    if (candidateIsSocket(selectedItem)) {
      connectionData.A.socketId = selectedItem.socket.def.id;
      connectionData.aDirection =
        selectedItem.socket.def.direction === "bidirectional"
          ? undefined
          : selectedItem.socket.def.direction;
      connectionData.A.componentId = selectedItem.component.def.id;

      if (inputARef.value) {
        inputARef.value.searchString = selectedItem.label;
      }
    }

    if (candidateIsProp(selectedItem)) {
      connectionData.A.attributePath = selectedItem.propPath;
      connectionData.aDirection = "input";
      connectionData.A.componentId = selectedItem.component.def.id;

      if (inputARef.value) {
        inputARef.value.searchString = selectedItem.label;
      }
    }

    activeSide.value = "b";
  } else {
    const selectedItem = fullBSideList.value[highlightedIndex.value];
    if (!selectedItem) return;
    if (candidateIsSocket(selectedItem)) {
      if (connectionData.A.socketId) {
        connectionData.B.socketId = selectedItem.socket.def.id;
        connectionData.aDirection = invertDirection(
          selectedItem.socket.def.direction === "bidirectional"
            ? undefined
            : selectedItem.socket.def.direction,
        );
        connectionData.B.componentId = selectedItem.component.def.id;
      } else {
        connectionData.aDirection =
          selectedItem.socket.def.direction === "bidirectional"
            ? undefined
            : selectedItem.socket.def.direction;

        connectionData.A.componentId = selectedItem.component.def.id;
        if (inputARef.value && inputBRef.value) {
          inputBRef.value.searchString = inputARef.value.searchString;
          inputARef.value.searchString = selectedItem.label;
        }
        if (fixedDirection.value) {
          fixedDirection.value = invertDirection(fixedDirection.value);
        }
      }
    }

    if (candidateIsProp(selectedItem)) {
      if (connectionData.A.attributePath) {
        connectionData.B.attributePath = selectedItem.propPath;
        connectionData.B.componentId = selectedItem.component.def.id;
      }
    }
  }

  const [from, to] =
    connectionData.aDirection === "output"
      ? [connectionData.A, connectionData.B]
      : [connectionData.B, connectionData.A];

  if (from.componentId && from.socketId && to.componentId && to.socketId) {
    componentsStore.CREATE_COMPONENT_CONNECTION(
      { componentId: from.componentId, socketId: from.socketId },
      { componentId: to.componentId, socketId: to.socketId },
    );
    // FIXME select both components when we show edges between the components that are selected
    componentsStore.eventBus.emit("setSelection", [to.componentId]);

    close();
  }

  // Create connection
  if (
    from.componentId &&
    from.attributePath &&
    to.componentId &&
    to.attributePath
  ) {
    componentsStore.UPDATE_COMPONENT_ATTRIBUTES(to.componentId, {
      [to.attributePath]: {
        $source: {
          component: from.componentId,
          path: from.attributePath,
        },
      },
    });
    close();
  }

  highlightedIndex.value = 0;
  focusOnInput();
};

// Modal Mgmt
function open(initialState: ConnectionMenuData) {
  highlightedIndex.value = 0;
  doneOpening.value = false;
  activeSide.value = "a";
  connectionData.A = {};
  connectionData.B = {};

  if (inputBRef.value) {
    inputBRef.value.searchString = "";
  }
  let initialASearch = "";
  let initialBSearch = "";

  let aSocketSelected = false;
  const initialComponentA =
    componentsStore.allComponentsById[initialState.A.componentId || ""];
  const initialComponentB =
    componentsStore.allComponentsById[initialState.B.componentId || ""];
  if (initialComponentA) {
    storedInitialComponent.value = initialComponentA;
    const componentViews =
      viewsStore.viewNamesByComponentId[initialComponentA.def.id];
    if (componentViews && componentViews.length > 0) {
      const componentView = componentViews.includes(
        viewsStore.selectedView?.name ?? "",
      )
        ? viewsStore.selectedView?.name
        : componentViews[0];

      if (componentView) {
        initialASearch += `${componentView}/`;
      }
    }

    initialASearch += `${initialComponentA.def.schemaName}/${initialComponentA.def.displayName}/`;

    const filteredSockets = initialComponentA.sockets.filter(
      (s) => !s.def.isManagement,
    );
    const initialSocket =
      filteredSockets.length === 1
        ? filteredSockets[0]
        : filteredSockets.find((s) => s.def.id === initialState.A.socketId);
    if (initialSocket) {
      aSocketSelected = true;
      initialASearch += `${initialSocket.def.label}`;
    }
    if (initialState.A.attributePath) {
      const strippedPath = initialState.A.attributePath.replace(/\//, "");
      aSocketSelected = true;
      initialASearch += strippedPath;
    }
  }
  if (initialComponentB) {
    const componentViews =
      viewsStore.viewNamesByComponentId[initialComponentB.def.id];
    if (componentViews && componentViews.length > 0) {
      const componentView = componentViews.includes(
        viewsStore.selectedView?.name ?? "",
      )
        ? viewsStore.selectedView?.name
        : componentViews[0];

      if (componentView) {
        initialBSearch += `${componentView}/`;
      }
    }

    initialBSearch += `${initialComponentB.def.schemaName}/${initialComponentB.def.displayName}/`;

    const filteredSockets = initialComponentB.sockets.filter(
      (s) => !s.def.isManagement,
    );
    const initialSocket =
      filteredSockets.length === 1
        ? filteredSockets[0]
        : filteredSockets.find((s) => s.def.id === initialState.B.socketId);
    if (initialSocket) {
      initialBSearch += `${initialSocket.def.label}`;
    }
    if (initialState.B.attributePath) {
      const strippedPath = initialState.B.attributePath.replace(/\//, "");
      initialBSearch += strippedPath;
    }
  }

  startingSearchStringA.value = initialASearch;
  startingSearchStringB.value = initialBSearch;

  if (initialState.aDirection && (initialASearch || initialBSearch)) {
    // can only have a fixed direction for a starting component
    fixedDirection.value = initialState.aDirection;
  } else {
    fixedDirection.value = undefined;
  }

  modalRef.value?.open();

  const finishOpening = setInterval(async () => {
    if (
      featureFlagsStore.PROPS_TO_PROPS_CONNECTIONS &&
      !loadedPropConnections.value
    ) {
      return;
    }
    if (!inputARef.value || !inputBRef.value) {
      return;
    }
    clearInterval(finishOpening);
    inputARef.value.searchString = initialASearch;
    inputBRef.value.searchString = initialBSearch;

    if (aSocketSelected) {
      // This makes sure we selected the right item from the list before selecting it
      for (const i in fullASideList.value) {
        const item = fullASideList.value[i];
        if (item && item.label === initialASearch) {
          highlightedIndex.value = Number(i);
          break;
        }
      }

      processHighlighted();
    }

    activeInputRef.value.value?.focus();
    doneOpening.value = true;
  }, 1);
}

function onClose() {
  storedInitialComponent.value = undefined;
}

function close() {
  modalRef.value?.close();
}

defineExpose({ open, close });
</script>

<script lang="ts">
export const generateUniquePathKey = (
  label: string,
  socket: DiagramSocketData,
) => `${label}-${socket.def.direction}-${socket.uniqueKey}`;
</script>
