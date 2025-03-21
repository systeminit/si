<template>
  <div class="flex flex-col basis-1/2 h-full">
    <div
      v-if="listItems.length"
      :class="
        clsx(
          'flex flex-col grow px-2xs gap-2xs py-3xs min-h-0 overflow-auto',
          !active && 'text-neutral-400',
        )
      "
    >
      <div
        v-for="(item, index) in listItems"
        :key="index"
        :class="
          clsx(
            'flex gap-xs align-middle items-center',
            active &&
              index === localHighlightedIndex &&
              'bg-action-600 text-white',
            active &&
              index !== localHighlightedIndex &&
              'hover:bg-action-200 hover:text-black',
            !active &&
              item.socket.def.id !== selectedSocket?.def.id &&
              'hover:bg-neutral-400 hover:text-black',
            index !== localHighlightedIndex &&
              item.socket.def.id === selectedSocket?.def.id &&
              'bg-neutral-600 text-white',
            'rounded cursor-pointer',
            'py-xs px-2xs my-0.5',
          )
        "
        @click="emit('select', index)"
      >
        <Icon
          :name="
            item.socket.def.direction === 'output'
              ? 'output-socket'
              : 'input-socket'
          "
          size="sm"
        />

        <span>
          {{ item.socket.def.label }} on component
          {{ item.component.def.title }} ({{ item.component.def.schemaName }})
        </span>
      </div>
    </div>
    <div
      v-else
      class="flex flex-col align-middle justify-center grow text-center text-neutral-600"
    >
      No available sockets
    </div>
    <div class="w-full border-t-2 p-xs min-h-[64px]">
      <div v-if="selectedSocket">
        Component: {{ selectedSocket.parent.def.displayName }} ({{
          selectedSocket.parent.def.schemaName
        }}) <br />
        Socket: {{ selectedSocket?.def.label }} <br />
      </div>
      <div v-else-if="selectedComponent">
        Component: {{ selectedComponent.def.displayName }} ({{
          selectedComponent.def.schemaName
        }}) <br />
        Socket: None
      </div>
      <div v-else>Nothing selected.</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import clsx from "clsx";
import { Icon } from "@si/vue-lib/design-system";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramSocketData,
} from "../ModelingDiagram/diagram_types";

export type SocketListEntry = {
  component: DiagramNodeData | DiagramGroupData;
  socket: DiagramSocketData;
};

const props = defineProps({
  listItems: {
    type: Array as PropType<SocketListEntry[]>,
    default: [] as SocketListEntry[],
  },
  selectedComponent: {
    type: Object as PropType<DiagramNodeData | DiagramGroupData>,
  },
  selectedSocket: { type: Object as PropType<DiagramSocketData> },
  highlightedIndex: { type: Number },
  active: { type: Boolean },
});
const localHighlightedIndex = computed(() =>
  props.active ? props.highlightedIndex : undefined,
);

const emit = defineEmits<{
  (e: "select", index: number): void;
}>();
</script>
