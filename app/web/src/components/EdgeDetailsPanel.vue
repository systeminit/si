<template>
  <ScrollArea v-if="selectedEdge">
    <template #top>
      <SidebarSubpanelTitle label="Connection Details" icon="plug">
        <DetailsPanelMenuIcon
          v-if="!featureFlagsStore.SIMPLE_SOCKET_UI"
          :selected="menuSelected"
          @click="
            (e) => {
              emit('openMenu', e);
            }
          "
        />
      </SidebarSubpanelTitle>
      <div
        v-if="featureFlagsStore.SIMPLE_SOCKET_UI"
        class="p-xs border-b dark:border-neutral-600 flex flex-row"
      >
        <VButton
          class="grow"
          size="sm"
          label="Add Connection"
          icon="plus"
          variant="ghost"
          @click="addConnection"
        />
      </div>
    </template>

    <div
      v-for="connection in connections"
      :key="connection.fromSocket.uniqueKey"
      class="border-b dark:border-neutral-600 p-xs"
    >
      <Connection
        :connection="connection"
        :showMenu="featureFlagsStore.SIMPLE_SOCKET_UI"
      />
    </div>

    <Stack
      v-if="
        selectedEdge.changeStatus === 'deleted' &&
        !featureFlagsStore.SIMPLE_SOCKET_UI
      "
      class="px-xs py-sm"
    >
      <ErrorMessage icon="alert-triangle" tone="warning">
        This edge will be removed from your model when this change set is merged
      </ErrorMessage>
      <VButton
        tone="shade"
        variant="ghost"
        size="md"
        icon="trash-restore"
        label="Restore edge"
        @click="modelingEventBus.emit('restoreSelection')"
      />
    </Stack>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import {
  VButton,
  Stack,
  ErrorMessage,
  ScrollArea,
} from "@si/vue-lib/design-system";
import {
  ConnectionDirection,
  useComponentsStore,
} from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";
import Connection from "./Connection.vue";

defineProps({
  menuSelected: { type: Boolean },
});

const featureFlagsStore = useFeatureFlagsStore();
const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();
const modelingEventBus = componentsStore.eventBus;

const selectedEdge = computed(() => viewsStore.selectedEdge);

const connections = computed(() => {
  if (!selectedEdge.value) return [];
  const fromComponent =
    componentsStore.allComponentsById[selectedEdge.value.fromComponentId];
  const toComponent =
    componentsStore.allComponentsById[selectedEdge.value.toComponentId];
  if (!fromComponent || !toComponent) return [];

  if (featureFlagsStore.SIMPLE_SOCKET_UI && !selectedEdge.value.isManagement) {
    const allEdges = Object.values(componentsStore.rawEdgesById);
    const edgesToDisplay = allEdges.filter(
      (edge) =>
        edge.fromComponentId === fromComponent.def.id &&
        edge.toComponentId === toComponent.def.id &&
        !edge.isManagement,
    );
    const connections = edgesToDisplay.map((edge) => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const fromSocket = fromComponent.sockets.find(
        (socket) => socket.def.id === edge.fromSocketId,
      )!;
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const toSocket = toComponent.sockets.find(
        (socket) => socket.def.id === edge.toSocketId,
      )!;

      return {
        id: edge.id,
        changeStatus: edge.changeStatus,
        createdInfo: edge.createdInfo,
        deletedInfo: edge.deletedInfo,
        isManagement: !!edge.isManagement,
        fromSocket,
        toSocket,
      };
    });

    return connections;
  } else {
    const fromSocket = fromComponent.sockets.find(
      (socket) => socket.def.id === selectedEdge.value?.fromSocketId,
    );
    const toSocket = toComponent.sockets.find(
      (socket) => socket.def.id === selectedEdge.value?.toSocketId,
    );

    if (!fromSocket || !toSocket) return []; // this should not happen!

    const connection = {
      id: selectedEdge.value.id,
      changeStatus: selectedEdge.value.changeStatus,
      createdInfo: selectedEdge.value.createdInfo,
      deletedInfo: selectedEdge.value.deletedInfo,
      isManagement: !!selectedEdge.value.isManagement,
      toSocket,
      fromSocket,
    };
    return [connection];
  }
});

const addConnection = () => {
  if (!selectedEdge.value || !featureFlagsStore.SIMPLE_SOCKET_UI) return;

  const fromId = selectedEdge.value.fromComponentId;
  const toId = selectedEdge.value.toComponentId;

  const menuData = {
    aDirection: "output" as ConnectionDirection,
    A: { componentId: fromId },
    B: { componentId: toId },
  };
  modelingEventBus.emit("openConnectionsMenu", menuData);
};

const emit = defineEmits<{
  (e: "openMenu", mouse: MouseEvent): void;
}>();
</script>
