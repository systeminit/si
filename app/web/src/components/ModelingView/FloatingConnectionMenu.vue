<template>
  <Modal ref="popoverRef" size="max" title="Create Connection">
    <div
      class="flex flex-col border-2 h-[80vh]"
      @click="focusOnInput"
      @keydown.up.prevent="highlightPrev"
      @keydown.down.prevent="highlightNext"
      @keydown.enter.prevent="processHighlighted"
      @keydown.tab.prevent="toggleEditTarget"
    >
      <VormInput ref="inputRef" v-model="searchString" label="Search" noLabel />
      <div class="flex flex-row grow border-t-2">
        <!-- Socket A -->
        <ConnectionMenuSocketList
          :active="activeSide === 'a'"
          :highlightedIndex="highlightedIndex"
          :listItems="listAItems"
          :selectedSocket="selectedSocketA"
          class="border-r-2"
          @select="selectA"
        />
        <!-- Socket B -->
        <ConnectionMenuSocketList
          :active="activeSide === 'b'"
          :highlightedIndex="highlightedIndex"
          :listItems="listBItems"
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
} from "vue";
import {
  ConnectionMenuData,
  useComponentsStore,
} from "@/store/components.store";
import ConnectionMenuSocketList, {
  SocketListEntry,
} from "./ConnectionMenuSocketList.vue";

const popoverRef = ref<InstanceType<typeof Modal>>();
const inputRef = ref<InstanceType<typeof VormInput>>();

onMounted(() => {
  componentsStore.eventBus.on("openConnectionsMenu", open);
});

onUnmounted(() => {
  componentsStore.eventBus.off("openConnectionsMenu", open);
});

const componentsStore = useComponentsStore();

const searchString = ref("");
// const socketKind = ref<"input" | "output" | undefined>();

const listAItems = ref<SocketListEntry[]>([]);
const listBItems = ref<SocketListEntry[]>([]);
// const filteredListItems = computed(() =>
//   listItems.value
//   // listItems.value.filter((s) => s.def.label.toLowerCase().includes(searchString.value.toLowerCase()))
// )

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

const bDirection = computed(() => invertDirection(connectionData.aDirection));

const selectedSocketA = computed(() => {
  if (!connectionData.A.componentId || !connectionData.A.socketId) {
    return;
  }

  const component =
    componentsStore.allComponentsById[connectionData.A.componentId];

  if (!component) return;

  return component.sockets.find((s) => s.def.id === connectionData.A.socketId);
});

const activeSide = ref<"a" | "b">("a");
const toggleEditTarget = () => {
  activeSide.value = activeSide.value === "a" ? "b" : "a";
};

// TODO only show sockets that have matching annotations
watch(
  connectionData,
  (data) => {
    let AComponents = data.A.componentId
      ? _.compact([componentsStore.allComponentsById[data.A.componentId]])
      : _.values(componentsStore.allComponentsById);

    if (data.B.componentId) {
      AComponents = AComponents.filter((c) => c.def.id !== data.B.componentId);
    }

    let aSockets = _.flatten(
      AComponents?.map((c) =>
        c.sockets
          .filter((s) => !s.def.isManagement)
          .map((s) => ({
            component: c,
            socket: s,
          })),
      ),
    );

    if (data.aDirection) {
      aSockets = aSockets.filter(
        ({ socket }) => socket.def.direction === data.aDirection,
      );
    }

    listAItems.value = aSockets;

    let BComponents = data.B.componentId
      ? _.compact([componentsStore.allComponentsById[data.B.componentId]])
      : _.values(componentsStore.allComponentsById);

    if (data.A.componentId) {
      BComponents = BComponents.filter((c) => c.def.id !== data.A.componentId);
    }

    let bSockets = _.flatten(
      BComponents?.map((c) =>
        c.sockets.map((s) => ({
          component: c,
          socket: s,
        })),
      ),
    );

    if (bDirection.value) {
      bSockets = bSockets.filter(
        ({ socket }) => socket.def.direction === bDirection.value,
      );
    }

    listBItems.value = bSockets;

    // const componentA = componentsStore.allComponentsById[data.A.componentId];
    //
    // if (!componentA) {
    //   listAItems.value = [];
    //   return;
    // }
    //
    // if (!data.B.componentId) {
    //   if (!data.A.socketId) {
    //     const sockets = data.A.direction ? componentA.sockets.filter((s) => s.def.direction === data.A.direction) : componentA.sockets;
    //
    //     listAItems.value = sockets.map(s => ({
    //       component: componentA,
    //       socket: s,
    //     }));
    //
    //   } else {
    //     // TODO filter component Bs to only have components that could match the selected socket
    //     listAItems.value = _.values(componentsStore.allComponentsById)
    //       .filter(c => c.def.id !== data.A.componentId)
    //       .map(c => ({
    //       component: c,
    //       socket: undefined,
    //     }));
    //   }

    //   return;
    // }
    //
    //
    // const componentB = componentsStore.allComponentsById[data.B.componentId];
    //
    // if (!componentB) {
    //   listAItems.value = [];
    //   return;
    // }
    //
    // if (!data.A.socketId) {
    //   // TODO list only sockets that would match between components A and B
    //   const sockets = data.A.direction ? componentA.sockets.filter((s) => s.def.direction === data.A.direction) : componentA.sockets;
    //
    //   listAItems.value = sockets.map(s => ({
    //     component: componentA,
    //     socket: s,
    //   }));
    //
    //   return;
    // }
    //
    //
    // if (data.B.socketId) {
    //   // TODO something when A and B sockets are selected
    //   listAItems.value = [];
    //   const [from, to] = data.A.direction === "output" ?
    //     [
    //       { componentId: data.A.componentId, socketId: data.A.socketId},
    //       { componentId: data.B.componentId, socketId: data.B.socketId},
    //     ] :
    //     [
    //       { componentId: data.B.componentId, socketId: data.B.socketId},
    //       { componentId: data.A.componentId, socketId: data.A.socketId},
    //     ];
    //
    //   componentsStore.CREATE_COMPONENT_CONNECTION(from, to);
    //   // FIXME select both components when we show edges between the components that are selected
    //   componentsStore.eventBus.emit("setSelection", [data.A.componentId]);
    //
    //   close();
    //   return;
    // }
    //
    // // TODO list only sockets that would match socket A
    // listAItems.value = componentB.sockets
    //   .map(s => ({
    //     component: componentB,
    //     socket: s,
    //   }));
  },
  { immediate: true },
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

const selectA = (index: number) => {
  highlightedIndex.value = index;
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
  popoverRef.value?.open();

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
  popoverRef.value?.close();
}

const isOpen = computed(() => popoverRef.value?.isOpen);

defineExpose({ open, close, isOpen });
</script>
