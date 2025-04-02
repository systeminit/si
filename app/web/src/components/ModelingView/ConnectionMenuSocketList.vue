<template>
  <div
    :class="
      clsx(
        'basis-1/2 h-full min-h-0 ',
        active && 'bg-neutral-800',
        socketToShow?.uniqueKey ? 'children:h-1/2' : 'children:h-full',
      )
    "
  >
    <div
      v-if="listItems.length"
      ref="scrollRef"
      :class="
        clsx(
          'flex flex-col px-2xs gap-2xs py-3xs min-h-0 overflow-auto relative',
          !active && 'text-neutral-400',
        )
      "
    >
      <div
        v-for="item in socketList"
        :key="item.index"
        :data-index="item.index"
        :class="
          clsx(
            'flex gap-xs align-middle items-center',
            active &&
              item.index === localHighlightedIndex &&
              'bg-action-600 text-white',
            active &&
              item.index !== localHighlightedIndex &&
              'hover:bg-action-200 hover:text-black',
            !active &&
              props.listItems[item.index]!.socket.def.id !== selectedSocket?.def.id &&
              'hover:bg-neutral-400 hover:text-black',
            item.index !== localHighlightedIndex &&
            props.listItems[item.index]!.socket.def.id === selectedSocket?.def.id &&
              'bg-neutral-600 text-white',
            'rounded cursor-pointer',
            'py-xs px-2xs my-0.5',
          )
        "
        :style="{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '100%',
          height: `${item.size}px`,
          transform: `translateY(${item.start}px)`,
        }"
        @click="emit('select', item.index)"
      >
        <Icon
          :class="clsx(props.listItems[item.index]!.socket.def.direction === 'output' && 'rotate-180')"
          :name="
            props.listItems[item.index]!.socket.def.direction === 'output'
              ? 'output-socket'
              : 'input-socket'
          "
          size="sm"
        />
        <span>
          <template
            v-for="(part, partIndex) in props.listItems[item.index]!.label.split('')"
            :key="partIndex"
          >
            <span v-if="part === '/'" class="text-neutral-400"> / </span>
            <b
              v-else-if="props.listItems[item.index]!.labelHighlights?.has(partIndex)"
              >{{ part }}</b
            >
            <span v-else>{{ part }}</span>
          </template>
        </span>
      </div>
    </div>
    <div
      v-else
      class="flex flex-col align-middle justify-center grow text-center text-neutral-600"
    >
      No available sockets
    </div>
    <div v-if="socketToShow?.uniqueKey" class="w-full border-t-2 min-h-0">
      <CodeEditor
        v-if="socketToShow.value"
        :id="`func-${socketToShow.uniqueKey}`"
        v-model="socketToShow.value"
        :recordId="''"
        disabled
        json
      />
      <div
        v-else
        class="flex flex-col h-full align-middle justify-center grow text-center"
      >
        &lt;EMPTY&gt;
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, reactive, ref, watchEffect } from "vue";
import clsx from "clsx";
import { Icon } from "@si/vue-lib/design-system";
import { useVirtualizer } from "@tanstack/vue-virtual";
import CodeEditor from "@/components/CodeEditor.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramSocketData,
} from "../ModelingDiagram/diagram_types";

export type SocketListEntry = {
  component: DiagramNodeData | DiagramGroupData;
  socket: DiagramSocketData;
  label: string;
  labelHighlights?: Set<number>;
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
  highlightedSocket: { type: Object as PropType<DiagramSocketData> },
  active: { type: Boolean },
});
const localHighlightedIndex = computed(() =>
  props.active ? props.highlightedIndex : undefined,
);

const scrollRef = ref<HTMLDivElement>();

const virtualList = useVirtualizer({
  count: props.listItems.length,
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  getScrollElement: () => scrollRef.value!,
  estimateSize: () => 37,
  overscan: 3,
});
const socketList = computed(() => virtualList.value.getVirtualItems());

const socketToShow = reactive<{ uniqueKey?: string; value?: string }>({});

watchEffect(() => {
  if (props.active && props.highlightedSocket) {
    socketToShow.uniqueKey = props.highlightedSocket.uniqueKey;
    socketToShow.value = JSON.stringify(
      props.highlightedSocket.def.value ?? undefined,
      null,
      2,
    );
  } else if (!props.active && props.selectedSocket) {
    socketToShow.uniqueKey = props.selectedSocket.uniqueKey;
    socketToShow.value = JSON.stringify(
      props.selectedSocket.def.value ?? undefined,
      null,
      2,
    );
  } else {
    socketToShow.uniqueKey = undefined;
    socketToShow.value = undefined;
  }
});

const emit = defineEmits<{
  (e: "select", index: number): void;
}>();
</script>
