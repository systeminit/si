<template>
  <ScrollArea v-if="selectedEdge">
    <template #top>
      <SidebarSubpanelTitle icon="plug" label="Connection Details">
        <DetailsPanelMenuIcon
          v-if="!featureFlagsStore.SIMPLE_SOCKET_UI"
          :selected="menuSelected"
          @click="
            (e: MouseEvent) => {
              emit('openMenu', e);
            }
          "
        />
      </SidebarSubpanelTitle>
      <div v-if="featureFlagsStore.SIMPLE_SOCKET_UI" class="p-xs border-b dark:border-neutral-600 flex flex-row">
        <VButton class="grow" icon="plus" label="Add Connection" size="sm" variant="ghost" @click="addConnection" />
      </div>
    </template>

    <div
      v-for="connection in connections"
      :key="connection.uniqueKeyForVue"
      class="border-b dark:border-neutral-600 p-xs"
    >
      <Connection :connection="connection" :showMenu="featureFlagsStore.SIMPLE_SOCKET_UI" />
    </div>

    <Stack v-if="selectedEdge.changeStatus === 'deleted' && !featureFlagsStore.SIMPLE_SOCKET_UI" class="px-xs py-sm">
      <ErrorMessage icon="alert-triangle" tone="warning">
        This edge will be removed from your model when this change set is merged
      </ErrorMessage>
      <VButton
        icon="trash-restore"
        label="Restore edge"
        size="md"
        tone="shade"
        variant="ghost"
        @click="modelingEventBus.emit('restoreSelection')"
      />
    </Stack>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { VButton, Stack, ErrorMessage, ScrollArea } from "@si/vue-lib/design-system";
import { ConnectionDirection, useComponentsStore } from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { Edge, isSocketEdge } from "@/api/sdf/dal/component";
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
  const fromComponent = componentsStore.allComponentsById[selectedEdge.value.fromComponentId];
  const toComponent = componentsStore.allComponentsById[selectedEdge.value.toComponentId];
  if (!fromComponent || !toComponent) return [];

  let edgesToDisplay: Edge[];
  if (featureFlagsStore.SIMPLE_SOCKET_UI && !selectedEdge.value.isManagement) {
    const allEdges = Object.values(componentsStore.rawEdgesById);
    edgesToDisplay = allEdges.filter(
      (edge) =>
        edge.fromComponentId === fromComponent.def.id &&
        edge.toComponentId === toComponent.def.id &&
        !edge.isManagement,
    );
  } else {
    edgesToDisplay = [selectedEdge.value];
  }
  return edgesToDisplay.map((edge) => {
    if (isSocketEdge(edge)) {
      return {
        ...edge,

        fromSocket: fromComponent.sockets.find((socket) => isSocketEdge(edge) && socket.def.id === edge.fromSocketId)!,

        toSocket: toComponent.sockets.find((socket) => isSocketEdge(edge) && socket.def.id === edge.toSocketId)!,
        uniqueKeyForVue: `${edge.fromSocketId}-${edge.toSocketId}`,
      };
    } else {
      // It's a subscription edge, then.
      return {
        ...edge,
        uniqueKeyForVue: `${edge.fromAttributePath}-${edge.toAttributePath}`,
      };
    }
  });
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
