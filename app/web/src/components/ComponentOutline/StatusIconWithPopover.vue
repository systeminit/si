<template>
  <div
    :class="
      clsx(
        'p-2xs rounded',
        popoverRef?.isOpen
          ? 'bg-action-300 dark:bg-action-400'
          : 'hover:bg-action-200 dark:hover:bg-action-300',
      )
    "
    @mouseenter="setHover(true)"
    @mouseleave="setHover(false)"
    @click="popoverRef.open"
  >
    <StatusIndicatorIcon
      :type="type"
      :status="status"
      :size="size"
      :tone="isHovered || popoverRef?.isOpen ? 'shade' : undefined"
    />
    <Popover
      ref="popoverRef"
      anchorAlignY="top"
      anchorDirectionX="right"
      :fixedPosition="popoverPosition"
    >
      <slot />
    </Popover>
  </div>
</template>

<script setup lang="ts">
import { ref, PropType } from "vue";
import { IconSizes } from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import clsx from "clsx";
import StatusIndicatorIcon, {
  IconType,
} from "@/components/StatusIndicatorIcon.vue";
import Popover from "../Popover.vue";

const props = defineProps({
  type: { type: String as PropType<IconType>, required: true },
  status: { type: String },
  size: { type: String as PropType<IconSizes> },
  popoverPosition: { type: Object as PropType<{ x: number; y: number }> },
});

const isHovered = ref(false);

function setHover(v: boolean) {
  isHovered.value = v;
}

const popoverRef = ref();
</script>
