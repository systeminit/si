<template>
  <div
    :class="
      clsx(
        'flex gap-xs align-middle items-center',
        active &&
          highlighted &&
          themeClasses('bg-action-200', 'bg-action-800 text-white'),
        active && [
          themeClasses('outline-action-500', 'outline-action-300'),
          'hover:outline -outline-offset-1 hover:outline-1',
        ],
        !active &&
          !selected && [
            'hover:outline -outline-offset-1 hover:outline-1',
            themeClasses(
              'hover:outline-neutral-500',
              'hover:outline-neutral-400',
            ),
          ],
        !highlighted &&
          selected && [
            'text-shade-0',
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
      height: `50px`,
      transform: `translateY(${item.start}px)`,
    }"
    @click="emit('select', item.index)"
  >
    <Icon
      :class="
        clsx('flex-none', themeClasses('text-neutral-500', 'text-neutral-400'))
      "
      :name="icon"
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
      {{ typeText }}
    </div>
    <div v-if="filteringBySearchString" class="flex flex-row grow items-center">
      <TruncateWithTooltip
        ref="lastPartRef"
        :lineClamp="3"
        class="text-xs font-bold flex-1 basis-1/2"
      >
        {{ lastPart }}
      </TruncateWithTooltip>
      <TruncateWithTooltip
        ref="breadcrumbRef"
        :class="
          clsx(
            'text-2xs flex-1 basis-1/2 leading-tight break-all',
            themeClasses('text-neutral-600', 'text-neutral-300'),
          )
        "
        :lineClamp="3"
      >
        {{ breadcrumb }}
      </TruncateWithTooltip>
    </div>
    <TruncateWithTooltip v-else :lineClamp="3" class="text-xs leading-tight">
      <span v-for="(part, partIndex) in labelParts" :key="partIndex">{{
        part
      }}</span>
    </TruncateWithTooltip>
    <div
      v-if="highlighted"
      :class="
        clsx(
          'flex flex-row flex-none gap-3xs items-center text-2xs',
          !filteringBySearchString && 'ml-auto',
        )
      "
    >
      <TextPill tighter variant="key">
        {{ controlScheme === "arrows" ? "Right" : "Tab" }}
      </TextPill>
      <div class="leading-snug">to select</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { computed, PropType, ref } from "vue";
import {
  Icon,
  themeClasses,
  TruncateWithTooltip,
  TextPill,
} from "@si/vue-lib/design-system";
import { VirtualItem } from "@tanstack/vue-virtual";
import {
  ConnectionCandidateListEntry,
  candidateIsSocket,
  candidateIsProp,
} from "./ConnectionMenuCandidateList.vue";

const props = defineProps({
  entry: {
    type: Object as PropType<ConnectionCandidateListEntry>,
    required: true,
  },
  item: { type: Object as PropType<VirtualItem>, required: true },
  active: { type: Boolean },
  highlighted: { type: Boolean },
  selected: { type: Boolean },
  filteringBySearchString: { type: String },
  controlScheme: { type: String, default: "arrows" },
});

const typeText = computed(() => {
  if (candidateIsProp(props.entry)) return "Prop";

  if (!candidateIsSocket(props.entry)) return "?";

  return props.entry.socket?.def.direction === "input" ? "Input" : "Ouput";
});

const icon = computed(() => {
  if (candidateIsSocket(props.entry)) {
    return props.entry.socket?.def.direction === "input"
      ? "input-connection"
      : "output-connection";
  }

  return "cursor";
});

const labelParts = computed(() => {
  return props.entry.label.split(/(\/)/);
});
const breadcrumbParts = computed(() => {
  return labelParts.value.slice(0, labelParts.value.length - 1);
});
const breadcrumb = computed(() => breadcrumbParts.value.join(""));
const lastPart = computed(() => {
  return labelParts.value[labelParts.value.length - 1];
});

const emit = defineEmits<{
  (e: "select", index: number): void;
}>();

const lastPartRef = ref<InstanceType<typeof TruncateWithTooltip>>();
const breadcrumbRef = ref<InstanceType<typeof TruncateWithTooltip>>();

const highlightRanges = computed(() => {
  const ranges = [] as Range[];
  const lastPartEl = lastPartRef.value?.$el as HTMLDivElement | undefined;
  const breadcrumbEl = breadcrumbRef.value?.$el as HTMLDivElement | undefined;
  const lastPartText = lastPartEl?.childNodes[1] as Text | undefined;
  const breadcrumbText = breadcrumbEl?.childNodes[1] as Text | undefined;

  if (
    !lastPartText ||
    !breadcrumbText ||
    !props.entry.labelHighlights ||
    breadcrumb.value !== breadcrumbText.wholeText ||
    lastPart.value !== lastPartText.wholeText
  ) {
    return [];
  }

  const sortedIndexes = [...props.entry.labelHighlights];
  sortedIndexes.sort((a, b) => a - b);

  const lastPartHighlights = [] as number[];
  const breadcrumbHighlights = [] as number[];
  sortedIndexes.forEach((i) => {
    if (i < breadcrumb.value.length) {
      breadcrumbHighlights.push(i);
    } else {
      lastPartHighlights.push(i - breadcrumb.value.length);
    }
  });

  const makeRanges = (highlightIndexes: number[], text: Text) => {
    let currentRange;
    let previousIndex = -1;
    for (const i of highlightIndexes) {
      if (currentRange) {
        if (previousIndex !== i - 1) {
          currentRange.setEnd(text, previousIndex + 1);
          ranges.push(currentRange);
          currentRange = new Range();
          currentRange.setStart(text, i);
        }
      } else {
        currentRange = new Range();
        currentRange.setStart(text, i);
      }
      previousIndex = i;
    }
    if (currentRange) {
      currentRange.setEnd(text, previousIndex + 1);
      ranges.push(currentRange);
    }
  };
  makeRanges(lastPartHighlights, lastPartText);
  makeRanges(breadcrumbHighlights, breadcrumbText);

  return ranges;
});

defineExpose({ highlightRanges });
</script>

<style lang="less">
::highlight(fuzzy-search-highlight) {
  color: white; // shade-0
  background-color: #2f80ed !important; // action-500
}
</style>
