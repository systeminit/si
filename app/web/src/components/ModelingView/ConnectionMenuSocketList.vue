<template>
  <div class="flex flex-col basis-1/2 h-full">
    <div
      :class="
        clsx(
          'flex flex-col grow px-2xs gap-2xs py-3xs',
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
              'bg-action-400 text-white',
            active &&
              index !== localHighlightedIndex &&
              'hover:bg-action-300 hover:text-black',
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
    <div class="w-full border-t-2 p-xs min-h-[64px]">
      <div v-if="selectedSocket">
        {{ selectedSocket?.def.id }}
        {{ selectedSocket?.def.label }}
      </div>
      <div v-else>Select the socket you would like to connect.</div>
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
  selectedSocket: { type: DiagramSocketData },
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
