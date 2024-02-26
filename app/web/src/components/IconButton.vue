<template>
  <div
    v-tooltip="{ content: tooltip, placement: 'left' }"
    :class="
      clsx(
        'rounded-md cursor-pointer',
        selected
          ? `text-shade-0 ${getToneBgColorClass(iconTone)}`
          : iconIdleTone
          ? hover
            ? getToneTextColorClass(iconTone)
            : getToneTextColorClass(iconIdleTone)
          : getToneTextColorClass(iconTone),
        !noBorderOnHover && 'border border-transparent p-[1px]',
        !noBorderOnHover && !selected && 'hover:border-action-500',
      )
    "
    @mouseover="onHover"
    @mouseleave="onEndHover"
    @click="onEndHover"
  >
    <Icon :name="iconShowing" :rotate="rotate" />
  </div>
</template>

<script lang="ts" setup>
import {
  Icon,
  IconNames,
  Tones,
  getToneBgColorClass,
  getToneTextColorClass,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType, computed, ref } from "vue";

const props = defineProps({
  icon: { type: String as PropType<IconNames>, required: true },
  iconHover: { type: String as PropType<IconNames> },
  iconTone: { type: String as PropType<Tones>, default: "action" },
  iconIdleTone: { type: String as PropType<Tones> },
  noBorderOnHover: { type: Boolean },
  selected: { type: Boolean },
  tooltip: { type: String },
  rotate: {
    type: String as PropType<"left" | "right" | "up" | "down">,
    default: undefined,
  },
});

const hover = ref(false);

const onHover = () => {
  hover.value = true;
};
const onEndHover = () => {
  hover.value = false;
};

const iconShowing = computed(() =>
  props.iconHover && hover.value && !props.selected
    ? props.iconHover
    : props.icon,
);
</script>
