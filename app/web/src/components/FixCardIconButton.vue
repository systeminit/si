<template>
  <div
    v-tooltip="{ content: tooltip, placement: 'left' }"
    :class="
      clsx(
        'rounded-md',
        selected
          ? 'text-shade-0 dark:bg-action-300 bg-action-500'
          : 'dark:text-action-300 text-action-500',
      )
    "
    @mouseover="onHover"
    @mouseleave="onEndHover"
  >
    <Icon :name="iconShowing" :rotate="rotate" />
  </div>
</template>

<script lang="ts" setup>
import { Icon, IconNames } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType, computed, ref } from "vue";

const props = defineProps({
  icon: { type: String as PropType<IconNames>, required: true },
  iconHover: { type: String as PropType<IconNames>, required: true },
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
  hover.value && !props.selected ? props.iconHover : props.icon,
);
</script>
