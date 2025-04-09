<template>
  <Modal
    ref="modalRef"
    size="max"
    title="Create a connection"
    hideExitButton
    :capitalizeTitle="false"
    @click="focusOnInput"
  >
    <template #titleIcons>
      <div class="text-xs cursor-pointer" @click="modalRef?.close()">
        <TextPill>ESC</TextPill> to exit
      </div>
    </template>
    <div
      class="flex flex-col h-[80vh] rounded-sm children:gap-sm"
      @click="focusOnInput"
      @keydown="debouncedListener"
    >
      <div class="flex flex-row w-full children:basis-1/2">
        <FloatingConnectionMenuInput
          ref="inputARef"
          :active="activeSide === 'a'"
          :focused="activeSide === 'a'"
        />
        <FloatingConnectionMenuInput
          ref="inputBRef"
          :active="activeSide === 'b'"
          :focused="activeSide === 'b'"
        />
      </div>
      <div class="flex flex-row grow min-h-0">
        <!-- Socket A -->
        <ConnectionMenuSocketList
          :doneLoading="doneOpening"
          :active="activeSide === 'a'"
          :highlightedIndex="highlightedIndex"
          :highlightedSocket="highlightedSocket"
          :listItems="listAItems"
          :selectedComponent="selectedComponentA"
          :selectedSocket="selectedSocketA"
          @select="(index: number) => selectAndProcess('a', index)"
        />
        <!-- Socket B -->
        <ConnectionMenuSocketList
          :doneLoading="doneOpening"
          :active="activeSide === 'b'"
          :highlightedIndex="highlightedIndex"
          :highlightedSocket="highlightedSocket"
          :listItems="listBItems"
          :selectedComponent="selectedComponentB"
          :selectedSocket="selectedSocketB"
          @select="(index: number) => selectAndProcess('b', index)"
        />
      </div>
    </div>
    <div
      class="flex flex-row w-full h-8 mt-sm items-center justify-end gap-sm text-xs"
    >
      <div class="flex flex-row gap-2xs items-center">
        <TextPill>Up</TextPill>
        <TextPill>Down</TextPill>
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
        <TextPill>Tab</TextPill>
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
        <TextPill>Shift + Tab</TextPill>
        <div>to switch back to the list on the left</div>
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
        <TextPill>Enter</TextPill>
        <div>to create a connection</div>
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { Modal, themeClasses } from "@si/vue-lib/design-system";
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
  ConnectionMenuData,
  generateSocketPaths,
  useComponentsStore,
} from "@/store/components.store";
import { DiagramSocketData } from "@/components/ModelingDiagram/diagram_types";
import TextPill from "@/components/TextPill.vue";
import { useViewsStore } from "@/store/views.store";
import ConnectionMenuSocketList, {
  SocketListEntry,
} from "./ConnectionMenuSocketList.vue";
import FloatingConnectionMenuInput from "./FloatingConnectionMenuInput.vue";

const modalRef = ref<InstanceType<typeof Modal>>();
const inputARef = ref<InstanceType<typeof FloatingConnectionMenuInput>>();
const inputBRef = ref<InstanceType<typeof FloatingConnectionMenuInput>>();
const searchStringA = computed(() => inputARef.value?.searchString);
const searchStringB = computed(() => inputBRef.value?.searchString);

const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();

const doneOpening = ref(false);

onMounted(() => {
  componentsStore.eventBus.on("openConnectionsMenu", open);
});

onUnmounted(() => {
  componentsStore.eventBus.off("openConnectionsMenu", open);
});

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

// this needs to compute based on allComponentsById changes to socket values
const highlightedSocket = computed(() => {
  const highlighted =
    activeSide.value === "a"
      ? listAItems.value[highlightedIndex.value]
      : listBItems.value[highlightedIndex.value];
  if (!highlighted) return undefined;
  return componentsStore.allComponentsById[
    highlighted.component.def.id
  ]?.sockets.find((s) => s.def.id === highlighted.socket.def.id);
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

// Generate the options on both sides
// TODO Break this into computed variables
const updateMenu = async () => {
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

  const [activeSearchString, otherSideSearchString] =
    activeSide.value === "a"
      ? [searchStringA.value, searchStringB.value]
      : [searchStringB.value, searchStringA.value];

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

  let matchedOtherSideSockets = deduplicatedOtherSideSockets;
  if (otherSideSearchString) {
    const fzf = new Fzf(deduplicatedOtherSideSockets, {
      casing: "case-insensitive",
      selector: (item) => item.label,
    });

    matchedOtherSideSockets = fzf.find(otherSideSearchString).map((e) => ({
      ...e.item,
      labelHighlights: e.positions,
    }));
  }

  activeSideList.value = activeSideSocketsWithPeers;
  otherSideList.value = matchedOtherSideSockets;
};
const debouncedUpdate = _.debounce(updateMenu, 20, {
  leading: true,
  trailing: true,
});
watch(activeSide, debouncedUpdate, { immediate: true });
watch(searchStringA, debouncedUpdate, { immediate: true });
watch(searchStringB, debouncedUpdate, { immediate: true });
watch(connectionData, debouncedUpdate, { immediate: true });
watch(componentsStore.diagramEdgesById, debouncedUpdate, { immediate: true });
watch(componentsStore.allComponentsById, debouncedUpdate, { immediate: true });

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
};
const debouncedListener = _.debounce(keyListener, 10, {
  leading: true,
  trailing: false,
});

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

    if (inputARef.value) {
      inputARef.value.searchString = selectedItem.label;
    }
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
  doneOpening.value = false;
  activeSide.value = "a";
  connectionData.A = {};
  connectionData.B = {};
  activeSide.value = "a";
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
  }

  modalRef.value?.open();

  const finishOpening = setInterval(async () => {
    if (!inputARef.value || !inputBRef.value) {
      return;
    }
    inputARef.value.searchString = initialASearch;
    inputBRef.value.searchString = initialBSearch;
    activeInputRef.value.value?.focus();
    if (aSocketSelected) {
      processHighlighted();
    }
    await updateMenu();
    doneOpening.value = true;
    clearInterval(finishOpening);
  }, 1);
}

function close() {
  modalRef.value?.close();
}

const isOpen = computed(() => modalRef.value?.isOpen);

defineExpose({ open, close, isOpen });
</script>
