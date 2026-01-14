<template>
  <div>
    <div v-if="isDevMode" class="pb-xs text-3xs italic opacity-30 break-all hidden">ID = {{ connection.id }}</div>
    <ConnectionEndCard
      v-if="isSocketConnection(connection)"
      :changeStatus="connection.changeStatus"
      :componentName="componentNames.from"
      :subjectLabel="connection.fromSocket.def.label"
      type="input-socket"
    />
    <ConnectionEndCard
      v-else
      :changeStatus="connection.changeStatus"
      :componentName="componentNames.from"
      :subjectLabel="connection.fromAttributePath"
      type="prop"
    />
    <div :class="clsx('_connection-label border-l-2', statusColors)">
      <div class="flex flex-row items-center">
        <DetailsPanelTimestamps
          :changeStatus="connection.changeStatus"
          :created="connection.createdInfo"
          :deleted="connection.deletedInfo"
          noMargin
        />
        <template v-if="showMenu">
          <DetailsPanelMenuIcon @click="openMenu" />
          <DropdownMenu ref="menuRef" :items="menuItems" variant="editor" />
        </template>
      </div>
    </div>
    <ConnectionEndCard
      v-if="isSocketConnection(connection)"
      :changeStatus="connection.changeStatus"
      :componentName="componentNames.to"
      :subjectLabel="connection.toSocket.def.label"
      type="input-socket"
    />
    <ConnectionEndCard
      v-else
      :changeStatus="connection.changeStatus"
      :componentName="componentNames.to"
      :subjectLabel="connection.toAttributePath"
      type="prop"
    />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType, ref } from "vue";
import clsx from "clsx";
import { DropdownMenu, DropdownMenuItemObjectDef, themeClasses } from "@si/vue-lib/design-system";
import { tw } from "@si/vue-lib";
import { isDevMode } from "@/utils/debug";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ActorAndTimestamp, AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { useComponentsStore } from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import { DiagramSocketData } from "./ModelingDiagram/diagram_types";
import ConnectionEndCard from "./ConnectionEndCard.vue";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

interface BaseConnection {
  id: string;
  changeStatus?: ChangeStatus;
  createdInfo?: ActorAndTimestamp;
  deletedInfo?: ActorAndTimestamp;
}

export interface SocketConnection extends BaseConnection {
  isManagement?: boolean;
  fromSocket: DiagramSocketData;
  toSocket: DiagramSocketData;
}
function isSocketConnection(connection: Connection): connection is SocketConnection {
  return "fromSocket" in connection;
}

export interface SubscriptionConnection extends BaseConnection {
  fromComponentId: ComponentId;
  fromAttributePath: AttributePath;
  toComponentId: ComponentId;
  toAttributePath: AttributePath;
}
function isSubscriptionConnection(connection: Connection): connection is SubscriptionConnection {
  return "fromAttributePath" in connection;
}

export type Connection = SocketConnection | SubscriptionConnection;

const props = defineProps({
  connection: { type: Object as PropType<Connection>, required: true },
  showMenu: { type: Boolean },
});

const menuRef = ref<InstanceType<typeof DropdownMenu>>();

const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();
const modelingEventBus = componentsStore.eventBus;

const componentNames = computed(() => {
  if (isSubscriptionConnection(props.connection)) {
    const toComponent = componentsStore.allComponentsById[props.connection.toComponentId];
    const fromComponent = componentsStore.allComponentsById[props.connection.fromComponentId];
    return {
      to: toComponent?.def.displayName ?? "?",
      from: fromComponent?.def.displayName ?? "?",
    };
  } else {
    // Is socket connection
    return {
      to: props.connection.toSocket.parent.def.displayName ?? "?",
      from: props.connection.fromSocket.parent.def.displayName ?? "?",
    };
  }
});

const openMenu = (e: MouseEvent) => {
  menuRef.value?.open(e);
};

const menuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  const disabled = false;
  // single selected edge
  items.push({
    label: "CONNECTION",
    header: true,
  });

  if (props.connection.changeStatus === "deleted") {
    items.push({
      label: "Restore",
      icon: "trash-restore",
      onSelect: triggerRestore,
      disabled,
    });
  } else {
    items.push({
      label: "Delete",
      shortcut: "⌫",
      icon: "trash",
      onSelect: triggerDelete,
      disabled,
    });
  }
  return items;
});

const triggerRestore = () => {
  viewsStore.setSelectedEdgeId(props.connection.id, viewsStore.selectedEdgeId);
  modelingEventBus.emit("restoreSelection");
};

const triggerDelete = () => {
  viewsStore.setSelectedEdgeId(props.connection.id, viewsStore.selectedEdgeId);
  modelingEventBus.emit("deleteSelection");
};

const statusColors = computed(() => {
  const unmodified = themeClasses(
    tw`border-shade-100 before:bg-shade-100 after:bg-shade-100`,
    tw`border-shade-0 before:bg-shade-0 after:bg-shade-0`,
  );
  if (!props.connection.changeStatus) return unmodified;
  const colors = {
    added: themeClasses(
      tw`border-success-500 before:bg-success-500 after:bg-success-500`,
      tw`border-success-400 before:bg-success-400 after:bg-success-400`,
    ),
    deleted: tw`border-destructive-500 before:bg-destructive-500 after:bg-destructive-500`,
    modified: tw`border-warning-400 before:bg-warning-400 after:bg-warning-400`,
    unmodified,
  };
  return colors[props.connection.changeStatus];
});
</script>

<style lang="less">
@socket-size: 10px;
._connection-label {
  padding: 8px;
  position: relative;
  z-index: 1;
  margin-left: 20px;

  &:before,
  &:after {
    content: "";
    width: @socket-size;
    height: @socket-size;
    border-radius: 100%;
    display: block;
    position: absolute;
    margin-left: (-@socket-size / 2 - 1);
    left: 0;
  }

  &::before {
    top: 0;
    margin-top: -(@socket-size / 2);
  }

  &::after {
    margin-bottom: -(@socket-size / 2);
    bottom: 0;
  }
}
</style>
