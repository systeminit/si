<template>
  <div
    ref="panelRef"
    :class="
      clsx(
        'si-panel',
        `si-panel-${side}`,
        side === 'right' ? 'z-[15]' : 'z-20',
        'dark:text-white pointer-events-auto relative',
        isTopOrBottom
          ? 'border-shade-100 bg-white dark:bg-neutral-900'
          : 'dark:border-neutral-600 border-neutral-300 bg-white dark:bg-neutral-800',
        {
          left: 'border-r-2 ',
          right: 'border-l-2',
          top: 'border-b shadow-[0_4px_4px_0_rgba(0,0,0,0.15)]',
          bottom: 'border-t shadow-[0_-4px_4px_0_rgba(0,0,0,0.15)]',
        }[side],
        `${side}-0`,
        !resizing && 'transition-[width]',
      )
    "
    :style="{
      ...(resizeable && {
        [isTopOrBottom ? 'height' : 'width']: `${displaySize}px`,
      }),
    }"
  >
    <PanelResizingHandle
      v-if="resizeable"
      class="si-panel__resizer"
      :panelSide="side"
      :collapsed="collapsed"
      @resize-start="onResizeStart"
      @resize-move="onResizeMove"
      @resize-end="onResizeEnd"
      @resize-reset="resetSize"
      @collapse-toggle="collapseToggle"
    />
    <!-- We blank out the contents of the ResizeablePanel while it is collapsing or opening from collapse to prevent messiness with the elements inside -->
    <div
      :class="
        clsx(
          'si-panel__inner absolute w-full h-full flex flex-col',
          `si-panel__inner-${side}`,
          (collapsed || panelOpeningFromCollapse) && 'opacity-0',
          themeClasses('bg-shade-0', 'bg-neutral-800'),
        )
      "
    >
      <!-- most uses will just have a single child -->
      <slot />

      <!-- but here we give the option for 2 resizable child sections within -->
      <div
        v-if="$slots.subpanel1"
        ref="subpanel1Ref"
        :style="{
          height: disableSubpanelResizing
            ? 'auto'
            : `${displaySubpanelSplitPercent * 100}%`,
        }"
        :class="clsx('relative', !subpanelResizing && 'transition-[height]')"
      >
        <div class="relative overflow-hidden w-full h-full">
          <slot name="subpanel1" />
        </div>
        <PanelResizingHandle
          v-if="
            $slots.subpanel1 && $slots.subpanel2 && !disableSubpanelResizing
          "
          :panelSide="isTopOrBottom ? 'left' : 'top'"
          :class="isTopOrBottom ? 'h-full' : 'w-full'"
          :collapsed="subpanelCollapsed"
          @resize-start="onSubpanelResizeStart"
          @resize-move="onSubpanelResizeMove"
          @resize-end="onSubpanelResizeEnd"
          @resize-reset="onSubpanelResizeReset"
          @collapse-toggle="subpanelCollapseToggle"
        />
      </div>

      <div
        v-if="$slots.subpanel2"
        :style="{
          height: disableSubpanelResizing
            ? 'auto'
            : `${100 - displaySubpanelSplitPercent * 100}%`,
        }"
        :class="
          clsx(
            'grow relative overflow-hidden',
            !subpanelResizing && 'transition-[height]',
          )
        "
      >
        <slot name="subpanel2" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  PropType,
  ref,
} from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import PanelResizingHandle from "./PanelResizingHandle.vue";
import { themeClasses } from "../utils/theme_tools";

// This variable determines how long after uncollapsing the main panel the panel's content should show
// We hide the panel content while it is collapsing or uncollapsing to avoid the content inside from displaying in jenky ways
// The CSS animation takes ~150ms, so this number should be equal to or less than that number
const PANEL_COLLAPSE_HIDE_CONTENT_TIME = 140;

const props = defineProps({
  rememberSizeKey: { type: String, required: true },
  side: {
    type: String as PropType<"left" | "right" | "top" | "bottom">,
    required: true,
  },
  hidden: { type: Boolean },
  sizeClasses: { type: String, default: "" },
  resizeable: { type: Boolean, default: true },
  minSizeRatio: { type: Number, default: 0.1 },
  minSize: { type: Number, default: 200 },
  maxSizeRatio: { type: Number, default: 0.45 },
  maxSize: { type: Number },
  defaultSize: { type: Number, default: 320 },
  disableSubpanelResizing: { type: Boolean },
  defaultSubpanelSplit: { type: Number, default: 0.5 },
});

const APP_MINIMUM_WIDTH = 700; // APP_MINIMUM_WIDTH
const getWindowWidth = () => {
  if (window.innerWidth > APP_MINIMUM_WIDTH) return window.innerWidth;
  else return APP_MINIMUM_WIDTH;
};

const isTopOrBottom = computed(
  () => props.side === "top" || props.side === "bottom",
);

const panelRef = ref<HTMLDivElement>();
const currentSize = ref(0);
const displaySize = computed(() => {
  if (collapsed.value) return 0;
  else return currentSize.value;
});
const collapsed = ref(false);
const panelOpeningFromCollapse = ref(false);
const panelOpeningFromCollapseTimeout = ref<number>();

const setSize = (newSize: number) => {
  let finalSize = newSize;

  // TODO: make sure these checks don't conflict with each other

  if (props.minSize) {
    if (finalSize < props.minSize) finalSize = props.minSize;
  }
  if (props.minSizeRatio) {
    const limit =
      (isTopOrBottom.value ? window.innerHeight : getWindowWidth()) *
      props.minSizeRatio;
    if (finalSize < limit) finalSize = limit;
  }

  if (props.maxSize) {
    if (finalSize > props.maxSize) finalSize = props.maxSize;
  }
  if (props.maxSizeRatio) {
    const limit =
      (isTopOrBottom.value ? window.innerHeight : getWindowWidth()) *
      props.maxSizeRatio;

    if (finalSize > limit) finalSize = limit;
  }
  currentSize.value = finalSize;
  emit("sizeSet", finalSize);
  if (finalSize === props.defaultSize) {
    window.localStorage.removeItem(primarySizeLocalStorageKey.value);
  } else {
    window.localStorage.setItem(
      primarySizeLocalStorageKey.value,
      `${finalSize}`,
    );
  }
};

const emit = defineEmits<{
  (e: "sizeSet", width: number): void;
}>();

const maximize = () => {
  if (props.maxSize) {
    setSize(props.maxSize);
  } else if (props.maxSizeRatio) {
    const limit =
      (isTopOrBottom.value ? window.innerHeight : getWindowWidth()) *
      props.maxSizeRatio;
    setSize(limit);
  }
};

const primarySizeLocalStorageKey = computed(
  () => `${props.rememberSizeKey}-size`,
);
const subpanelSplitLocalStorageKey = computed(
  () => `${props.rememberSizeKey}-split`,
);

const resizing = ref(true);
const beginResizeValue = ref(0);
const onResizeStart = () => {
  beginResizeValue.value = currentSize.value;
  resizing.value = true;
};

const onResizeMove = (delta: number) => {
  const adjustedDelta =
    props.side === "right" || props.side === "bottom" ? delta : -delta;
  setSize(beginResizeValue.value + adjustedDelta);
};

const onResizeEnd = () => {
  resizing.value = false;
};

const resetSize = (useDefaultSize = true) => {
  if (props.defaultSize && useDefaultSize) {
    setSize(props.defaultSize);
  }
};

const collapseSet = (collapse: boolean) => {
  collapsed.value = collapse;
  if (!collapsed.value) {
    panelOpeningFromCollapse.value = true;
    panelOpeningFromCollapseTimeout.value = window.setTimeout(() => {
      emit("sizeSet", displaySize.value);
      panelOpeningFromCollapse.value = false;
    }, PANEL_COLLAPSE_HIDE_CONTENT_TIME);
  } else {
    panelOpeningFromCollapseTimeout.value = window.setTimeout(() => {
      emit("sizeSet", displaySize.value);
      panelOpeningFromCollapse.value = false;
    }, PANEL_COLLAPSE_HIDE_CONTENT_TIME);
  }
};

const collapseToggle = () => {
  collapseSet(!collapsed.value);
};

const onWindowResize = () => {
  // may change the size because min/max ratio of window size may have changed
  if (props.resizeable) setSize(currentSize.value);
  else currentSize.value = props.defaultSize;
};

const debounceForResize = _.debounce(onWindowResize, 20);
const windowResizeObserver = new ResizeObserver(debounceForResize);

// subpanel resizing
const subpanelSplitPercent = ref(props.defaultSubpanelSplit);
const displaySubpanelSplitPercent = computed(() => {
  if (subpanelCollapsed.value) return 100;
  else return subpanelSplitPercent.value;
});

onMounted(async () => {
  const storedSplit = window.localStorage.getItem(
    subpanelSplitLocalStorageKey.value,
  );
  if (storedSplit) subpanelSplitPercent.value = parseFloat(storedSplit);
  // We have resizing be true for just the first tick to avoid the panels animating into place when the DOM reloads
  await nextTick(() => {
    resizing.value = false;
  });
});

const subpanel1Ref = ref();
const subpanelResizing = ref(false);
const subpanelCollapsed = ref(false);
const subpanelOpeningFromCollapse = ref(false);
const subpanelOpeningFromCollapseTimeout = ref<number>();
const totalAvailableSize = ref(0);
const subpanel1SizePx = computed(
  () => totalAvailableSize.value * subpanelSplitPercent.value,
);
let subpanelResizeStartPanel1Size: number;
function onSubpanelResizeStart() {
  subpanelResizing.value = true;
  const boundingRect = panelRef.value?.getBoundingClientRect();
  if (!boundingRect) return;
  totalAvailableSize.value = isTopOrBottom.value
    ? boundingRect.width
    : boundingRect.height;
  subpanelResizeStartPanel1Size = subpanel1SizePx.value;
}
function onSubpanelResizeMove(delta: number) {
  const newPanel1SizePx = subpanelResizeStartPanel1Size - delta;
  setSubpanelSplit(newPanel1SizePx / totalAvailableSize.value);
}
function onSubpanelResizeReset() {
  setSubpanelSplit(props.defaultSubpanelSplit);
}
function setSubpanelSplit(newSplitPercent: number) {
  if (newSplitPercent < 0.2) {
    subpanelSplitPercent.value = 0.2;
  } else if (newSplitPercent > 0.8) {
    subpanelSplitPercent.value = 0.8;
  } else {
    subpanelSplitPercent.value = newSplitPercent;
  }

  if (subpanelSplitPercent.value === props.defaultSubpanelSplit) {
    window.localStorage.removeItem(subpanelSplitLocalStorageKey.value);
  } else {
    window.localStorage.setItem(
      subpanelSplitLocalStorageKey.value,
      `${subpanelSplitPercent.value}`,
    );
  }
}
function onSubpanelResizeEnd() {
  subpanelResizing.value = false;
}
function subpanelCollapseSet(collapse: boolean) {
  subpanelCollapsed.value = collapse;
  if (!subpanelCollapsed.value) {
    subpanelOpeningFromCollapse.value = true;
    subpanelOpeningFromCollapseTimeout.value = window.setTimeout(() => {
      subpanelOpeningFromCollapse.value = false;
    }, PANEL_COLLAPSE_HIDE_CONTENT_TIME);
  }
}
function subpanelCollapseToggle() {
  subpanelCollapseSet(!subpanelCollapsed.value);
}

onMounted(() => {
  if (props.resizeable) {
    const storedSize = window.localStorage.getItem(
      primarySizeLocalStorageKey.value,
    );
    if (storedSize) {
      setSize(parseInt(storedSize));
    } else {
      setSize(props.defaultSize);
    }
  } else {
    window.localStorage.removeItem(primarySizeLocalStorageKey.value);
  }

  windowResizeObserver.observe(document.body);
});

onBeforeUnmount(() => {
  windowResizeObserver.unobserve(document.body);
  if (panelOpeningFromCollapseTimeout.value) {
    clearTimeout(panelOpeningFromCollapseTimeout.value);
  }
});

defineExpose({
  setSize,
  maximize,
  resetSize,
  maxSize: props.maxSize,
  collapseToggle,
  collapseSet,
  collapsed,
  subpanelCollapseToggle,
  subpanelCollapseSet,
  subpanelCollapsed,
});
</script>
