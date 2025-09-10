<template>
  <div class="flex flex-col gap-2xs">
    <div
      v-tooltip="{
        content: 'Copied!',
        triggers: [],
        shown: showTooltip,
        autoHide: true,
        theme: 'instant-show',
      }"
      :class="
        clsx(
          'flex flex-row gap-sm items-center justify-between border rounded-sm cursor-pointer select-none',
          !expandable && 'h-full',
          prompt ? 'p-sm' : 'p-xs',
          themeClasses(
            'bg-neutral-100 border-neutral-400',
            'bg-neutral-900 border-neutral-600',
          ),
          !hoverSubIcon &&
            themeClasses(
              'hover:bg-neutral-300 active:bg-neutral-400',
              'hover:bg-neutral-600 active:bg-neutral-700',
            ),
        )
      "
      @click="copyText(text)"
    >
      <Icon
        v-if="expandable"
        v-tooltip="expanded ? 'Collapse' : 'Expand'"
        :name="expanded ? 'chevron-down' : 'chevron-right'"
        :class="
          clsx(
            'flex-none',
            themeClasses(
              'hover:bg-neutral-400 active:bg-neutral-500',
              'hover:bg-neutral-600 active:bg-neutral-700',
            ),
          )
        "
        @click.stop.prevent="onClickExpand"
        @mouseenter="onMouseEnterSubIcon"
        @mouseleave="onMouseLeaveSubIcon"
      />
      <div
        :class="
          clsx(
            'flex-grow text-sm',
            breakWords && 'break-all',
            !prompt &&
              'overflow-hidden text-ellipsis whitespace-nowrap py-2xs font-mono',
          )
        "
      >
        {{ text }}
      </div>
      <Icon v-tooltip="'Copy'" name="copy" class="flex-none" size="sm" />
    </div>
    <div
      v-if="expandable && expanded"
      :class="
        clsx(
          'rounded-sm border p-xs text-sm leading-4 break-all font-mono',
          themeClasses(
            'bg-neutral-100 border-neutral-400',
            'bg-neutral-900 border-neutral-600',
          ),
        )
      "
    >
      {{ text }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ref } from "vue";

defineProps({
  text: { type: String, required: true },
  prompt: Boolean,
  expandable: Boolean,
  breakWords: Boolean,
});

const expanded = ref(false);
const showTooltip = ref(false);
const hoverSubIcon = ref(false);

const copyText = (text: string) => {
  navigator.clipboard.writeText(text);
  showTooltip.value = true;
  setTimeout(() => {
    showTooltip.value = false;
  }, 300);
  emit("copied");
};

const onMouseEnterSubIcon = () => {
  hoverSubIcon.value = true;
};

const onMouseLeaveSubIcon = () => {
  hoverSubIcon.value = false;
};

const onClickExpand = () => {
  expanded.value = !expanded.value;
  onMouseLeaveSubIcon();
};

const emit = defineEmits<{
  (e: "copied"): void;
}>();
</script>
