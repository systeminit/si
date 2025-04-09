<template>
  <div
    :class="
      clsx(
        'basis-1/2 h-full min-h-0 border-x-2 border-b-2 p-xs ',
        active && themeClasses('bg-neutral-100', 'bg-neutral-800'),
        socketToShow?.uniqueKey && doneLoading
          ? 'children:h-1/2'
          : 'children:h-full',
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
          ref="socketListItemsRef"
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
          <span class="line-clamp-2 text-xs flex-1">
            <!-- TODO(Wendy) - this should probably not just truncate, we need to see it! -->
            <template
              v-for="(part, partIndex) in props.listItems[
                item.index
              ]!.label.split('')"
              :key="partIndex"
            >
              <span
                v-if="part === '/'"
                :class="themeClasses('text-neutral-500', 'text-neutral-400')"
              >
                /
              </span>
              <b
                v-else-if="
                  props.listItems[item.index]!.labelHighlights?.has(partIndex)
                "
                >{{ part }}</b
              >
              <span v-else>{{ part }}</span>
            </template>
          </span>
          <div
            v-if="item.index === localHighlightedIndex"
            class="flex flex-row flex-none gap-3xs items-center text-2xs"
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
      No available sockets
    </div>
    <div v-if="socketToShow?.uniqueKey && doneLoading" class="w-full min-h-0">
      <CodeEditor
        v-if="socketToShow.value"
        :id="`func-${socketToShow.uniqueKey}`"
        v-model="socketToShow.value"
        :recordId="''"
        disabled
        noVim
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
import { computed, PropType, reactive, ref, watch, watchEffect } from "vue";
import clsx from "clsx";
import { Icon, LoadingMessage, themeClasses } from "@si/vue-lib/design-system";
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
    estimateSize: () => 37,
    overscan: 3,
  };
});

const virtualList = useVirtualizer(virtualizerOptions);

const socketList = computed(() => virtualList.value.getVirtualItems());
const socketListSize = computed(() => virtualList.value.getTotalSize());

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

const socketListItemsRef = ref<InstanceType<typeof HTMLDivElement>[]>([]);
watch(
  () => localHighlightedIndex.value,
  (i) => {
    if (socketListItemsRef.value && i !== undefined) {
      socketListItemsRef.value[i]?.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
      });
    }
  },
);
</script>
