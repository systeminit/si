<template>
  <div
    :class="
      clsx(
        'basis-1/2 h-full min-h-0 border-x-2 border-b-2 p-xs ',
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
          'flex flex-col px-2xs gap-2xs py-3xs min-h-0 overflow-auto',
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
        <div
          v-for="item in socketList"
          :key="item.index"
          :data-index="item.index"
          :class="
            clsx(
              'flex gap-xs align-middle items-center',
              active &&
                item.index === localHighlightedIndex &&
                themeClasses('bg-action-200', 'bg-action-800 text-white'),
              active && [
                themeClasses('outline-action-500', 'outline-action-300'),
                'hover:outline -outline-offset-1 hover:outline-1',
              ],
              !active &&
                props.listItems[item.index]!.socket.def.id !==
                  selectedSocket?.def.id && [
                  'hover:outline -outline-offset-1 hover:outline-1',
                  themeClasses(
                    'hover:outline-neutral-500',
                    'hover:outline-neutral-400',
                  ),
                ],
              item.index !== localHighlightedIndex &&
                props.listItems[item.index]!.socket.def.id ===
                  selectedSocket?.def.id && [
                  'text-white',
                  themeClasses('bg-action-500', 'bg-action-600'),
                ],
              'cursor-pointer',
              'py-xs px-2xs my-3xs',
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
            :class="
              clsx(
                'flex-none',
                themeClasses('text-neutral-500', 'text-neutral-400'),
              )
            "
            :name="
              props.listItems[item.index]!.socket.def.direction === 'output'
                ? 'output-connection'
                : 'input-connection'
            "
            size="sm"
          />
          <div
            :class="
              clsx(
                'w-8 text-2xs flex-none',
                themeClasses('text-neutral-500', 'text-neutral-400'),
              )
            "
          >
            {{
              props.listItems[item.index]!.socket.def.direction === "output"
                ? "Output"
                : "Input"
            }}
          </div>
          <template v-if="filteringBySearchString">
            <TruncateWithTooltip
              class="text-xs font-bold flex-none max-w-[40%]"
              :lineClamp="3"
            >
              {{ lastPart[item.index] }}
            </TruncateWithTooltip>
            <TruncateWithTooltip
              :class="
                clsx(
                  'text-2xs flex-shrink ml-auto min-w-0 leading-tight',
                  themeClasses('text-neutral-600', 'text-neutral-300'),
                )
              "
              :lineClamp="3"
            >
              <template
                v-for="(part, partIndex) in breadcrumbParts[item.index]"
                :key="partIndex"
                >{{ part }}</template
              >
            </TruncateWithTooltip>
          </template>
          <TruncateWithTooltip
            v-else
            :lineClamp="3"
            class="text-xs leading-tight"
          >
            <span
              v-for="(part, partIndex) in labelParts[item.index]"
              :key="partIndex"
              >{{ part }}</span
            >
          </TruncateWithTooltip>
          <div
            v-if="item.index === localHighlightedIndex"
            :class="
              clsx(
                'flex flex-row flex-none gap-3xs items-center text-2xs',
                !filteringBySearchString && 'ml-auto',
              )
            "
          >
            <TextPill tighter>Tab</TextPill>
            <div class="leading-snug">to select</div>
          </div>
        </div>
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
import {
  Icon,
  LoadingMessage,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import { useVirtualizer } from "@tanstack/vue-virtual";
import CodeEditor from "@/components/CodeEditor.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramSocketData,
} from "../ModelingDiagram/diagram_types";
import TextPill from "../TextPill.vue";

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
const labelParts = computed(() => {
  return props.listItems.map((item) => {
    return item.label.split(/(\/)/);
  });
});
const breadcrumbParts = computed(() => {
  return labelParts.value.map((item) => {
    return item.slice(0, item.length - 1);
  });
});
const lastPart = computed(() => {
  return labelParts.value.map((item) => {
    return item[item.length - 1];
  });
});

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
</script>
