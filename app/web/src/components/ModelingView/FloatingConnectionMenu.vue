<template>
  <Modal ref="modalRef" size="max" title="Create Connection">
    <div
      class="flex flex-col border-2 h-[80vh]"
      @click="focusOnInput"
      @keydown.up.prevent="highlightPrev"
      @keydown.down.prevent="highlightNext"
      @keydown.enter.prevent="processHighlighted"
      @keydown.tab.prevent="toggleEditTarget"
    >
      <VormInput ref="inputRef" v-model="searchString" label="Search" noLabel />
      <div class="flex flex-row grow border-t-2 min-h-0">
        <!-- Socket A -->
        <ConnectionMenuSocketList
          :active="activeSide === 'a'"
          :highlightedIndex="highlightedIndex"
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
import {
  ConnectionMenuData,
  useComponentsStore,
} from "@/store/components.store";
import { DiagramSocketData } from "@/components/ModelingDiagram/diagram_types";
import ConnectionMenuSocketList, {
  SocketListEntry,
} from "./ConnectionMenuSocketList.vue";

const modalRef = ref<InstanceType<typeof Modal>>();
const inputRef = ref<InstanceType<typeof VormInput>>();

onMounted(() => {
  componentsStore.eventBus.on("openConnectionsMenu", open);
});

onUnmounted(() => {
  componentsStore.eventBus.off("openConnectionsMenu", open);
});

const componentsStore = useComponentsStore();

const searchString = ref("");

const listAItems = ref<SocketListEntry[]>([]);
const listBItems = ref<SocketListEntry[]>([]);

const focusOnInput = () => {
  nextTick(() => {
    inputRef.value?.focus();
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

const activeSide = ref<"a" | "b">("a");
const toggleEditTarget = () => {
  activeSide.value = activeSide.value === "a" ? "b" : "a";
};

// TODO Break this into computed variables
watchEffect(() => {
  const [activeSideData, otherSideData] =
    activeSide.value === "a"
      ? [connectionData.A, connectionData.B]
      : [connectionData.B, connectionData.A];
  const [activeSideList, otherSideList] =
    activeSide.value === "a"
      ? [listAItems, listBItems]
      : [listBItems, listAItems];

  // I'm the active side
  // I need to run the peersFunc for *all my components*
  // If I can still estabilish connections, I'll be on my list

  const edges = _.values(componentsStore.diagramEdgesById);

  // Active side
  const componentsActiveSide = _.filter(
    activeSideData.componentId
      ? _.compact([
          componentsStore.allComponentsById[activeSideData.componentId],
        ])
      : _.values(componentsStore.allComponentsById),
    // Do not get the peer component, if one is selected
    (c) => !otherSideData.componentId || c.def.id !== otherSideData.componentId,
  );

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
        .filter((s) =>
          s.def.label
            .toLowerCase()
            .includes(searchString.value?.toLowerCase() ?? ""),
        )
        .map((s) => ({
          component: c,
          socket: s,
        })),
    ),
  );

  const peersFunction = componentsStore.possibleAndExistingPeerSocketsFn;
  const validPeers = _.flatten(
    activeSideSockets
      .map(({ component, socket }) =>
        peersFunction(socket.def, component.def.id).possiblePeers.map((p) => ({
          originatorSocket: socket,
          ...p,
        })),
      )
      .filter((peers) => peers.length > 0),
  )
    .filter((s) => !otherSideData.socketId || otherSideData.socketId === s.id)
    .filter(
      (s) =>
        !otherSideData.componentId ||
        otherSideData.componentId === s.componentId,
    )
    .map((s) => {
      const component = componentsStore.allComponentsById[s.componentId];

      if (!component) {
        return;
      }

      return {
        originatorSocket: s.originatorSocket,
        component,
        socket: new DiagramSocketData(component, s),
      };
    });

  // Remove the undefined
  const otherSideSockets = {} as Record<string, SocketListEntry>;
  const socketsWithPossiblePeers = new Set();
  for (const peer of _.compact(validPeers)) {
    otherSideSockets[peer.socket.uniqueKey] = peer;
    socketsWithPossiblePeers.add(peer.originatorSocket.uniqueKey);
  }

  activeSideList.value = activeSideSockets.filter((e) =>
    socketsWithPossiblePeers.has(e.socket.uniqueKey),
  );
  otherSideList.value = _.values(otherSideSockets);
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
  } else {
    toggleEditTarget();
  }

  highlightedIndex.value = 0;
  focusOnInput();
};

// Modal Mgmt
function open(initialState: ConnectionMenuData) {
  modalRef.value?.open();

  searchString.value = "";
  activeSide.value = "a";
  connectionData.aDirection = initialState.aDirection;
  connectionData.A = initialState.A;
  connectionData.B = initialState.B;

  // For some reason nextTick does not work but this does and UX feels fine
  setTimeout(() => {
    inputRef.value?.focus();
  }, 100);
}

function close() {
  modalRef.value?.close();
}

const isOpen = computed(() => modalRef.value?.isOpen);

defineExpose({ open, close, isOpen });
</script>
