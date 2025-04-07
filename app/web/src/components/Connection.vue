<template>
  <div>
    <div
      v-if="isDevMode"
      class="pb-xs text-3xs italic opacity-30 break-all hidden"
    >
      ID = {{ connection.id }}
    </div>
    <SocketCard
      :socket="connection.fromSocket"
      outputSocket
      :changeStatus="connection.changeStatus"
    />
    <div
      :class="
        clsx(
          '_connection-label',
          'border-l-2',
          connection.changeStatus
            ? {
                added: 'border-success-400',
                deleted: 'border-destructive-500',
                modified: 'border-warning-400',
                unmodified: 'border-shade-0',
              }[connection.changeStatus]
            : 'border-shade-0',
          connection.changeStatus
            ? {
                added: 'before:bg-success-400 after:bg-success-400',
                deleted: 'before:bg-destructive-500 after:bg-destructive-500',
                modified: 'before:bg-warning-400 after:bg-warning-400',
                unmodified: 'before:bg-shade-0 after:bg-shade-0',
              }[connection.changeStatus]
            : 'before:bg-shade-0 after:bg-shade-0',
        )
      "
    >
      <div class="flex flex-row items-center">
        <DetailsPanelTimestamps
          noMargin
          :changeStatus="connection.changeStatus"
          :created="connection.createdInfo"
          :deleted="connection.deletedInfo"
        />
        <template v-if="showMenu">
          <DetailsPanelMenuIcon @click="openMenu" />
          <DropdownMenu ref="menuRef" :items="menuItems" variant="editor" />
        </template>
      </div>
    </div>
    <SocketCard
      :socket="connection.toSocket"
      :changeStatus="connection.changeStatus"
    />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType, ref } from "vue";
import clsx from "clsx";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { isDevMode } from "@/utils/debug";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ActorAndTimestamp } from "@/api/sdf/dal/component";
import { useComponentsStore } from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import { DiagramSocketData } from "./ModelingDiagram/diagram_types";
import SocketCard from "./SocketCard.vue";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

export type Connection = {
  id: string;
  changeStatus?: ChangeStatus;
  createdInfo: ActorAndTimestamp;
  deletedInfo?: ActorAndTimestamp;
  fromSocket: DiagramSocketData;
  toSocket: DiagramSocketData;
  isManagement: boolean;
};

const props = defineProps({
  connection: { type: Object as PropType<Connection>, required: true },
  showMenu: { type: Boolean },
});

const menuRef = ref<InstanceType<typeof DropdownMenu>>();

const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();
const modelingEventBus = componentsStore.eventBus;

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
