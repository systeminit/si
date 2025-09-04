<!-- THIS COMPONENT IS DEPRECATED, DO NOT USE IT ANYMORE!-->

<template>
  <div
    ref="mainDivRef"
    v-tooltip="{
      content: computedLoading && loadingTooltip ? loadingTooltip : tooltip,
      placement: tooltipPlacement,
    }"
    :tabindex="tabIndex"
    :class="
      clsx(
        'cursor-pointer rounded p-3xs group/iconbutton',
        disableClicking
          ? `opacity-50 ${getToneTextColorClass(iconToneDefault)}`
          : [
              selectedOrActive
                ? [
                    iconTone === 'shade'
                      ? themeClasses(
                          'text-shade-0 bg-shade-100',
                          'text-shade-100 bg-shade-0',
                        )
                      : [
                          'text-shade-0',
                          iconBgActiveTone
                            ? getToneBgColorClass(iconBgActiveTone)
                            : getToneBgColorClass(iconTone),
                        ],
                  ]
                : [
                    hover || hovered
                      ? [
                          iconIdleTone
                            ? getToneTextColorClass(iconTone)
                            : getToneTextColorClass(iconToneDefault),
                          'bg-neutral-200 dark:bg-neutral-600',
                        ]
                      : getToneTextColorClass(iconToneDefault),
                  ],
            ],
      )
    "
    @mousedown="startActive"
    @mouseleave="onEndHover"
    @mouseover="onHover"
    @mouseup="endActive"
    @click.stop="onClick"
  >
    <Icon
      class="group-hover/iconbutton:scale-110"
      :name="iconShowing"
      :rotate="rotate"
      :size="size"
    />

    <!-- Slot is for dropdown menus or modals to go in only! -->
    <slot />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { Placement } from "floating-vue";
import { PropType, computed, ref } from "vue";
import Icon from "../icons/Icon.vue";
import { IconNames } from "../icons/icon_set";
import {
  getToneBgColorClass,
  getToneTextColorClass,
  Tones,
} from "../utils/color_utils";
import { SpacingSizes } from "../utils/size_utils";
import { ApiRequestStatus } from "../../pinia";
import { themeClasses } from "../utils/theme_tools";

const props = defineProps({
  size: { type: String as PropType<SpacingSizes>, default: "md" },
  icon: { type: String as PropType<IconNames>, required: true },
  iconHover: { type: String as PropType<IconNames> },
  iconTone: { type: String as PropType<Tones>, default: "action" },
  iconIdleTone: { type: String as PropType<Tones> },
  iconBgActiveTone: { type: String as PropType<Tones> },
  selected: { type: Boolean },
  hovered: { type: Boolean },
  tooltip: { type: String },
  tooltipPlacement: { type: String as PropType<Placement>, default: "left" },
  rotate: {
    type: String as PropType<"left" | "right" | "up" | "down">,
    default: undefined,
  },
  disabled: { type: Boolean },
  loading: { type: Boolean },
  loadingIcon: { type: String as PropType<IconNames>, default: "loader" },
  loadingTooltip: { type: String },
  requestStatus: {
    type: [Boolean, Object] as PropType<false | ApiRequestStatus>, // can be false if passing 'someCondition && status'
  },
  tabIndex: Number,
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

const iconToneDefault = computed(() =>
  props.iconIdleTone ? props.iconIdleTone : props.iconTone,
);

const mainDivRef = ref<HTMLDivElement>();

defineExpose({ startActive, endActive, onHover, onEndHover, mainDivRef });
</script>
