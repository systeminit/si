<template>
  <div
    :class="
      clsx(
        'h-full min-h-0 border-x-2 border-b-2 p-xs pt-2xs ',
        active && themeClasses('bg-neutral-100', 'bg-neutral-800'),
        showCodeViewer ? 'children:h-1/2' : 'children:h-full',
      )
    "
  >
    <div v-if="!doneLoading" class="h-full flex flex-row items-center">
      <LoadingMessage message="Loading socket data..." />
    </div>
    <div
      v-else-if="listItems.length"
      ref="scrollRef"
      :class="
        clsx(
          'flex flex-col px-2xs gap-2xs py-3xs min-h-0 overflow-auto border',
          themeClasses('border-neutral-300', 'border-neutral-600'),
          !showCodeViewer && 'h-full',
          !active && themeClasses('text-neutral-500', 'text-neutral-400'),
        )
      "
    >
      <div
        :style="{
          height: `${socketListSize}px`,
          width: '100%',
          position: 'relative',
        }"
      >
        <ConnectionMenuSocketListItem
          v-for="item in socketList"
          ref="socketListItemsRef"
          :key="item.index"
          :data-index="item.index"
          :item="item"
          :entry="listItems[item.index]!"
          :highlighted="item.index === localHighlightedIndex"
          :active="active"
          :selected="listItems[item.index]!.socket.def.id === selectedSocket?.def.id"
          :filteringBySearchString="filteringBySearchString"
          :controlScheme="controlScheme"
          @select="(index) => emit('select', index)"
        />
      </div>
    </div>
    <div
      v-else
      class="flex flex-col align-middle justify-center grow text-center text-neutral-600"
    >
      No available sockets with possible connections
    </div>
    <div v-if="showCodeViewer" class="w-full min-h-0">
      <CodeEditor
        v-if="socketToShow.value"
        :id="`func-${socketToShow.uniqueKey}`"
        v-model="socketToShow.value"
        :recordId="''"
        disabled
        noVim
        json
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, PropType, reactive, ref, watch, watchEffect } from "vue";
import clsx from "clsx";
import { LoadingMessage, themeClasses } from "@si/vue-lib/design-system";
import { useVirtualizer } from "@tanstack/vue-virtual";
import CodeEditor from "@/components/CodeEditor.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramSocketData,
} from "../ModelingDiagram/diagram_types";
import ConnectionMenuSocketListItem from "./ConnectionMenuSocketListItem.vue";

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
  doneLoading: { type: Boolean },
  filteringBySearchString: { type: String },
  controlScheme: { type: String, default: "arrows" },
});
const localHighlightedIndex = computed(() =>
  props.active ? props.highlightedIndex : undefined,
);

const scrollRef = ref<HTMLDivElement>();

const virtualizerOptions = computed(() => {
  return {
    count: props.listItems.length,
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    getScrollElement: () => scrollRef.value!,
    estimateSize: () => 50,
    overscan: 3,
  };
});

const virtualList = useVirtualizer(virtualizerOptions);

const socketList = computed(() => virtualList.value.getVirtualItems());
const socketListSize = computed(() => virtualList.value.getTotalSize());

const socketToShow = reactive<{ uniqueKey?: string; value?: string }>({});

const showCodeViewer = computed(
  () => socketToShow?.uniqueKey && props.doneLoading && socketToShow.value,
);

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

watch(
  () => localHighlightedIndex.value,
  (i) => {
    if (i !== undefined) {
      virtualList.value.scrollToIndex(i, { align: "center" });
    }
  },
);

// In order to use the CSS Custom Highlight API with Typescript we have to use some workarounds -
// https://github.com/microsoft/TypeScript/issues/53003
declare class Highlight {
  add(range: Range): undefined;
  constructor(...range: Range[]);
}

// eslint-disable-next-line @typescript-eslint/no-namespace
declare namespace CSS {
  const highlights: Map<string, Highlight>;
}

// END WORKAROUND CODE

const socketListItemsRef =
  ref<InstanceType<typeof ConnectionMenuSocketListItem>[]>();

watchEffect(() => {
  if (
    !props.active ||
    !props.filteringBySearchString ||
    !socketListItemsRef.value
  )
    return;

  const allHighlightRanges = [] as Range[];

  socketListItemsRef.value.forEach((listItem) => {
    allHighlightRanges.push(...listItem.highlightRanges);
  });

  try {
    const fuzzySearchHighlight = new Highlight(...allHighlightRanges);
    CSS.highlights.set("fuzzy-search-highlight", fuzzySearchHighlight);
  } catch (e) {
    // eslint-disable-next-line no-console
    console.log("This browser does not support the CSS Custom Highlight API");
  }
});
</script>

<style lang="css">
::highlight(fuzzy-search-highlight) {
  background-color: red;
}
</style>
