<template>
  <Modal ref="modalRef" size="max" title="Create Connection">
    <div
      class="flex flex-col border-2 h-[80vh] rounded-sm"
      @click="focusOnInput"
      @keydown.up.prevent="highlightPrev"
      @keydown.down.prevent="highlightNext"
      @keydown.enter.prevent="processHighlighted"
      @keydown.tab.exact.prevent="processHighlighted"
      @keydown.shift.tab.prevent="undoASelection"
    >
      <div class="flex flex-row w-full children:basis-1/2">
        <VormInput
          ref="inputARef"
          v-model="searchStringA"
          class="border-r-2"
          label="Search"
          noLabel
        />
        <VormInput
          ref="inputBRef"
          v-model="searchStringB"
          label="Search"
          noLabel
        />
      </div>
      <div class="flex flex-row grow border-t-2 min-h-0">
        <!-- Socket A -->
        <ConnectionMenuSocketList
          :active="activeSide === 'a'"
          :highlightedIndex="highlightedIndex"
          :highlightedSocket="highlightedSocket"
          :listItems="listAItems"
          :selectedComponent="selectedComponentA"
          :selectedSocket="selectedSocketA"
          class="border-r-2"
          @select="(index:number) => selectAndProcess('a', index)"
        />
        <!-- Socket B -->
        <ConnectionMenuSocketList
          :active="activeSide === 'b'"
          :highlightedIndex="highlightedIndex"
          :highlightedSocket="highlightedSocket"
          :listItems="listBItems"
          :selectedComponent="selectedComponentB"
          :selectedSocket="selectedSocketB"
          @select="(index:number) => selectAndProcess('b', index)"
        />
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { VormInput, Modal } from "@si/vue-lib/design-system";
import {
  computed,
  nextTick,
  onMounted,
  onUnmounted,
  reactive,
  ref,
  watch,
  watchEffect,
} from "vue";
import { Fzf } from "fzf";
import {
  ConnectionMenuData,
  generateSocketPaths,
  useComponentsStore,
} from "@/store/components.store";
import { DiagramSocketData } from "@/components/ModelingDiagram/diagram_types";
import { useViewsStore } from "@/store/views.store";
import ConnectionMenuSocketList, {
  SocketListEntry,
} from "./ConnectionMenuSocketList.vue";

const modalRef = ref<InstanceType<typeof Modal>>();
const inputARef = ref<InstanceType<typeof VormInput>>();
const inputBRef = ref<InstanceType<typeof VormInput>>();

const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();

onMounted(() => {
  componentsStore.eventBus.on("openConnectionsMenu", open);
});

onUnmounted(() => {
  componentsStore.eventBus.off("openConnectionsMenu", open);
});

const searchStringA = ref("");
const searchStringB = ref("");
const activeInputRef = computed(() =>
  activeSide.value === "a" ? inputARef : inputBRef,
);

const listAItems = ref<SocketListEntry[]>([]);
const listBItems = ref<SocketListEntry[]>([]);

const focusOnInput = () => {
  nextTick(() => {
    activeInputRef.value.value?.focus();
  });
};

// Selection data
const connectionData = reactive<ConnectionMenuData>({
  aDirection: undefined,
  A: {
    componentId: undefined,
    socketId: undefined,
  },
  B: {
    componentId: undefined,
    socketId: undefined,
  },
});

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

const highlightedSocket = computed(
  () =>
    (activeSide.value === "a"
      ? listAItems.value[highlightedIndex.value]
      : listBItems.value[highlightedIndex.value]
    )?.socket,
);

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

// Generate the options on both sides
// TODO Break this into computed variables
watchEffect(() => {
  if (!modalRef.value?.isOpen) {
    return;
  }

  const [activeSideData, otherSideData] =
    activeSide.value === "a"
      ? [connectionData.A, connectionData.B]
      : [connectionData.B, connectionData.A];
  const [activeSideList, otherSideList] =
    activeSide.value === "a"
      ? [listAItems, listBItems]
      : [listBItems, listAItems];

  const activeSearchString =
    activeSide.value === "a" ? searchStringA.value : searchStringB.value;

  const edges = _.values(componentsStore.diagramEdgesById);

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
        // If socket is unary and has connection, skip it
        .filter((s) => {
          if (s.def.direction !== "input" || s.def.maxConnections === null) {
            return true;
          }

          const incomingConnections = edges.filter(
            (e) => e.toSocketKey === s.uniqueKey,
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
        }, [] as SocketListEntry[]),
    ),
  );

  const sortFn = (s1: SocketListEntry, s2: SocketListEntry) => {
    if (s1.socket.def.direction === s2.socket.def.direction) {
      if (s1.label < s2.label) {
        return -1;
      }
      if (s1.label > s2.label) {
        return 1;
      }
      return 0;
    }
    if (s1.socket.def.direction === "input") {
      return -1;
    } else {
      return 1;
    }
  };

  activeSideSockets.sort(sortFn);

  let matchedActiveSideSockets = activeSideSockets;

  if (activeSearchString) {
    const fzf = new Fzf(activeSideSockets, {
      casing: "case-insensitive",
      selector: (item) => item.label,
    });

    matchedActiveSideSockets = fzf.find(activeSearchString).map((e) => ({
      ...e.item,
      labelHighlights: e.positions,
    }));
  }

  // Get all valid peer sockets for the active sockets
  const peersFunction = componentsStore.possibleAndExistingPeerSocketsFn;
  type SocketListEntryWithOriginator = SocketListEntry & {
    originatorSocketLabel: string;
  };

  const validPeers = matchedActiveSideSockets
    .flatMap(({ component, socket, label }) =>
      peersFunction(socket.def, component.def.id).possiblePeers.map((p) => ({
        originatorSocketLabel: label,
        ...p,
      })),
    )
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
        originatorSocketLabel: s.originatorSocketLabel,
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
  const otherSideSockets = {} as Record<string, SocketListEntry>;
  const socketsWithPossiblePeers = new Set();
  for (const peer of _.compact(validPeers)) {
    otherSideSockets[peer.label] = peer;
    socketsWithPossiblePeers.add(peer.originatorSocketLabel);
  }

  const activeSideSocketsWithPeers = matchedActiveSideSockets.filter((e) =>
    socketsWithPossiblePeers.has(e.label),
  );

  const deduplicatedOtherSideSockets = _.values(otherSideSockets);

  deduplicatedOtherSideSockets.sort(sortFn);

  activeSideList.value = activeSideSocketsWithPeers;
  otherSideList.value = deduplicatedOtherSideSockets;
});

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

const highlightNext = () => {
  const upperLimit =
    (activeSide.value === "a"
      ? listAItems.value.length
      : listBItems.value.length) - 1;

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
    const selectedItem = listAItems.value[highlightedIndex.value];
    if (!selectedItem) return;
    connectionData.A.socketId = selectedItem.socket.def.id;
    connectionData.aDirection =
      selectedItem.socket.def.direction === "bidirectional"
        ? undefined
        : selectedItem.socket.def.direction;
    connectionData.A.componentId = selectedItem.component.def.id;

    searchStringA.value = selectedItem.label;
    activeSide.value = "b";
  } else {
    const selectedItem = listBItems.value[highlightedIndex.value];
    if (!selectedItem) return;

    connectionData.B.socketId = selectedItem.socket.def.id;
    connectionData.aDirection = invertDirection(
      selectedItem.socket.def.direction === "bidirectional"
        ? undefined
        : selectedItem.socket.def.direction,
    );

    connectionData.B.componentId = selectedItem.component.def.id;
  }

  if (
    connectionData.A.componentId &&
    connectionData.B.componentId &&
    connectionData.A.socketId &&
    connectionData.B.socketId
  ) {
    const [from, to] =
      connectionData.aDirection === "output"
        ? [
            {
              componentId: connectionData.A.componentId,
              socketId: connectionData.A.socketId,
            },
            {
              componentId: connectionData.B.componentId,
              socketId: connectionData.B.socketId,
            },
          ]
        : [
            {
              componentId: connectionData.B.componentId,
              socketId: connectionData.B.socketId,
            },
            {
              componentId: connectionData.A.componentId,
              socketId: connectionData.A.socketId,
            },
          ];

    componentsStore.CREATE_COMPONENT_CONNECTION(from, to);
    // FIXME select both components when we show edges between the components that are selected
    componentsStore.eventBus.emit("setSelection", [
      connectionData.A.componentId,
    ]);

    close();
  }

  highlightedIndex.value = 0;
  focusOnInput();
};

// Modal Mgmt
function open(initialState: ConnectionMenuData) {
  activeSide.value = "a";
  connectionData.A = {};
  connectionData.B = {};
  activeSide.value = "a";
  searchStringB.value = "";

  let initialASearch = "";
  let aSocketSelected = false;
  const initialComponent =
    componentsStore.allComponentsById[initialState.A.componentId || ""];
  if (initialComponent) {
    const componentViews =
      viewsStore.viewNamesByComponentId[initialComponent.def.id];
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

    initialASearch += `${initialComponent.def.schemaName}/${initialComponent.def.displayName}/`;

    const filteredSockets = initialComponent.sockets.filter(
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
  }

  searchStringA.value = initialASearch;

  modalRef.value?.open();
  // For some reason nextTick does not work but this does and UX feels fine
  setTimeout(() => {
    activeInputRef.value.value?.focus();
    if (aSocketSelected) {
      processHighlighted();
    }
  }, 100);
}

function close() {
  modalRef.value?.close();
}

const isOpen = computed(() => modalRef.value?.isOpen);

defineExpose({ open, close, isOpen });
</script>
