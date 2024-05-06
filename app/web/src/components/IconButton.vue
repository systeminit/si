<template>
  <!-- TODO(Wendy) - probably at some point IconButton should be merged with VButton -->
  <div
    v-tooltip="{
      content: computedLoading && loadingTooltip ? loadingTooltip : tooltip,
      placement: tooltipPlacement,
    }"
    :class="
      clsx(
        'cursor-pointer',
        variant === 'classic' && [
          'rounded-md',
          disableClicking
            ? 'border border-transparent p-[1px]'
            : [
                !noBorderOnHover && 'border border-transparent p-[1px]',
                !noBorderOnHover &&
                  !selectedOrActive &&
                  'hover:border-action-500',
                selectedOrActive
                  ? `text-shade-0 ${getToneBgColorClass(iconTone)}`
                  : iconIdleTone
                  ? hover
                    ? getToneTextColorClass(iconTone)
                    : getToneTextColorClass(iconIdleTone)
                  : getToneTextColorClass(iconTone),
              ],
        ],
        variant === 'simple' && [
          'rounded p-[2px]',
          disabled
            ? `opacity-50 ${getToneTextColorClass(iconTone)}`
            : [
                selectedOrActive
                  ? `text-shade-0 ${getToneBgColorClass(iconTone)}`
                  : getToneTextColorClass(iconTone),
                !selectedOrActive &&
                  hover &&
                  !disableClicking &&
                  (iconIdleTone
                    ? getToneTextColorClass(iconIdleTone)
                    : 'bg-neutral-200 dark:bg-neutral-600'),
              ],
        ],
      )
    "
    @mouseover="onHover"
    @mouseleave="onEndHover"
    @click.stop="onClick"
    @mousedown="startActive"
    @mouseup="endActive"
  >
    <Icon :name="iconShowing" :rotate="rotate" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  Icon,
  IconNames,
  Tones,
  getToneBgColorClass,
  getToneTextColorClass,
} from "@si/vue-lib/design-system";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import clsx from "clsx";
import { Placement } from "floating-vue";
import { PropType, computed, ref } from "vue";

export type IconButtonVariant = "classic" | "simple";

const props = defineProps({
  icon: { type: String as PropType<IconNames>, required: true },
  iconHover: { type: String as PropType<IconNames> },
  iconTone: { type: String as PropType<Tones>, default: "action" },
  iconIdleTone: { type: String as PropType<Tones> },
  noBorderOnHover: { type: Boolean },
  selected: { type: Boolean },
  tooltip: { type: String },
  tooltipPlacement: { type: String as PropType<Placement>, default: "left" },
  rotate: {
    type: String as PropType<"left" | "right" | "up" | "down">,
    default: undefined,
  },
  disabled: { type: Boolean },
  loading: { type: Boolean },
  loadingIcon: { type: String as PropType<IconNames> },
  loadingTooltip: { type: String },
  variant: { type: String as PropType<IconButtonVariant>, default: "classic" },
  requestStatus: {
    type: [Boolean, Object] as PropType<false | ApiRequestStatus>, // can be false if passing 'someCondition && status'
  },
});

const emit = defineEmits(["click"]);

const hover = ref(false);
const active = ref(false);
const computedLoading = computed(
  () => props.loading || !!_.get(props, "requestStatus.isPending"),
);
const selectedOrActive = computed(() => props.selected || active.value);
const disableClicking = computed(() => props.disabled || computedLoading.value);

const onHover = () => {
  hover.value = true;
};
const onEndHover = () => {
  hover.value = false;
  endActive();
};

const startActive = () => {
  if (!disableClicking.value) {
    active.value = true;
  }
};

const endActive = () => {
  active.value = false;
};

const onClick = (e: MouseEvent) => {
  onEndHover();
  if (!disableClicking.value) {
    emit("click", e);
  }
};

const iconShowing = computed(() => {
  if (props.loadingIcon && computedLoading.value) return props.loadingIcon;
  else
    return props.iconHover && hover.value && !props.selected
      ? props.iconHover
      : props.icon;
});
</script>
