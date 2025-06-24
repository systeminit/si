<template>
  <component
    :is="htmlTagOrComponentType"
    v-bind="dynamicAttrs"
    ref="component"
    v-tooltip="truncateRef?.tooltipActive ? truncateRef.tooltip : undefined"
    class="vbutton"
    :tabindex="tabIndex"
    :class="clsx(computedClasses, computedTextSize)"
    @click="clickHandler($event)"
  >
    <div class="vbutton__inner">
      <template v-if="computedLoading">
        <Icon class="vbutton__icon" :name="loadingIcon" />
        <span class="vbutton__text"> {{ loadingText }}</span>
      </template>
      <template v-else-if="showSuccess">
        <Icon class="vbutton__icon" :name="successIcon" />
        <span class="vbutton__text">
          <slot name="success">{{ successText }}</slot>
        </span>
      </template>

      <template v-else>
        <slot name="icon">
          <Icon
            v-if="icon"
            :class="
              clsx('vbutton__icon', props.iconClass ? props.iconClass : '')
            "
            :name="icon"
          />
        </slot>
        <TruncateWithTooltip
          v-if="truncateText"
          ref="truncateRef"
          class="py-xs"
        >
          <slot v-if="confirmClick && confirmFirstClickAt" name="confirm-click">
            |
            {{
              confirmClick === true
                ? "You sure? Click again to confirm"
                : confirmClick
            }}
          </slot>
          <slot v-else>{{ label }}</slot>
        </TruncateWithTooltip>
        <span v-else class="vbutton__text">
          <slot v-if="confirmClick && confirmFirstClickAt" name="confirm-click">
            |
            {{
              confirmClick === true
                ? "You sure? Click again to confirm"
                : confirmClick
            }}
          </slot>
          <slot v-else>{{ label }}</slot>
        </span>
        <slot name="iconRight">
          <Icon v-if="iconRight" class="vbutton__icon" :name="iconRight" />
        </slot>
        <slot name="pill">
          <TextPill v-if="pill" class="ml-2xs" mono>{{ pill }}</TextPill>
        </slot>
      </template>
    </div>
  </component>
</template>

<script lang="ts" setup>
import { ref, computed, onBeforeUnmount, watch, PropType } from "vue";
import { RouterLink } from "vue-router";
import * as _ from "lodash-es";
import clsx from "clsx";

import { ApiRequestStatus } from "../../pinia";

import Icon from "../icons/Icon.vue";
import { IconNames } from "../icons/icon_set";
import { Tones } from "../utils/color_utils";
import { useTheme } from "../utils/theme_tools";
import TextPill from "./TextPill.vue";
import TruncateWithTooltip from "./TruncateWithTooltip.vue";
import { tw } from "../../utils/tw-utils";

const SHOW_SUCCESS_DELAY = 2000;

type ButtonSizes = "2xs" | "xs" | "sm" | "md" | "lg" | "xl";
type ButtonVariants = "solid" | "ghost" | "soft" | "transparent";
type ButtonTones = Tones;

const props = defineProps({
  size: { type: String as PropType<ButtonSizes>, default: "md" },
  textSize: { type: String as PropType<ButtonSizes> },
  iconClass: { type: String },

  variant: { type: String as PropType<ButtonVariants>, default: "solid" },
  tone: { type: String as PropType<ButtonTones>, default: "action" },

  label: { type: String },

  icon: String as PropType<IconNames>,
  iconRight: String as PropType<IconNames>,
  href: String,
  linkToNamedRoute: String,
  linkTo: [String, Object],
  target: String,

  disabled: Boolean,
  loading: Boolean,
  loadingText: { type: String, default: "Loading..." },
  loadingIcon: { type: String as PropType<IconNames>, default: "loader" },

  requestStatus: {
    type: [Boolean, Object] as PropType<false | ApiRequestStatus>, // can be false if passing 'someCondition && status'
  },

  clickSuccess: { type: Boolean },
  successText: { type: String, default: "Success!" },
  iconSuccess: { type: String as PropType<IconNames> },

  confirmClick: { type: [Boolean, String] },

  submit: { type: Boolean },

  square: { type: Boolean },
  rounded: { type: Boolean },
  pill: { type: String, required: false },

  truncateText: { type: Boolean },

  hoverGlow: Boolean,
  tabIndex: Number,
});

const truncateRef = ref<InstanceType<typeof TruncateWithTooltip>>();

const emit = defineEmits(["click"]);

const successIcon = computed(() => {
  if (props.iconSuccess) return props.iconSuccess;
  return "check";
});

const htmlTagOrComponentType = computed(() => {
  if (props.href) return "a";
  if (props.linkTo || props.linkToNamedRoute) return RouterLink;
  return "button";
});
const htmlButtonType = computed(() => {
  if (htmlTagOrComponentType.value !== "button") return undefined;
  if (props.submit) return "submit";
  return "button";
});

// loading status can be passed in via loading prop or from requestStatus
const computedLoading = computed(
  () => props.loading || !!_.get(props, "requestStatus.isPending"),
);

// we use an object to do some dynamic bindings so we don't attach props that are not needed
const dynamicAttrs = computed(() => {
  return {
    // set the "to" prop if we are in router link mode
    ...(htmlTagOrComponentType.value === RouterLink && {
      to: props.linkToNamedRoute
        ? { name: props.linkToNamedRoute }
        : props.linkTo,
    }),

    // if we set href to undefined when in RouterLink mode, it doesn't set it properly
    ...(htmlTagOrComponentType.value === "a" && {
      href: props.href,
    }),

    // set the target when its a link/router link
    ...((htmlTagOrComponentType.value === RouterLink ||
      (htmlTagOrComponentType.value === "a" && props.target)) && {
      target: props.target,
    }),

    ...(htmlButtonType.value && {
      type: htmlButtonType.value,
    }),
  };
});

// watch request status and show a success message for a short time when request goes through
const showSuccess = ref(false);

watch(
  () => props.requestStatus,
  (newStatus, oldStatus, onInvalidate) => {
    // TODO: look over the reactivity / types here...

    // status object can change without values changing if using a keyed request status with a /*
    // that returns an object of keyed statuses
    if (_.isEqual(newStatus, oldStatus)) return;
    if (!newStatus) return;

    // toggle button to show a success message for a short period
    if (newStatus.isSuccess && props.successText) {
      showSuccess.value = true;
      const timeout = setTimeout(() => {
        showSuccess.value = false;
      }, SHOW_SUCCESS_DELAY);
      onInvalidate(() => clearTimeout(timeout));
    }
  },
  { deep: true },
);

// get the right type of timeout (some weirdness around NodeJS.Timeout)
type Timeout = ReturnType<typeof setTimeout>;

// confirm click functionality -- requires the user to click twice to confirm
// can be a nicer lightweight alternative to a modal
const confirmFirstClickAt = ref<Date | null>(null);
let confirmClickTimeout: Timeout;
let successClickTimeout: Timeout;
function clickHandler(e: MouseEvent) {
  if (props.confirmClick) {
    if (confirmFirstClickAt.value) {
      // check if the 2 clicks are super close together and ignore if they are
      // this is to help ignore some users who always double click everything
      if (+new Date() - +confirmFirstClickAt.value < 300) {
        return;
      }

      confirmFirstClickAt.value = null;
      clearTimeout(confirmClickTimeout);
      emit("click");
    } else {
      confirmFirstClickAt.value = new Date();
      confirmClickTimeout = setTimeout(() => {
        confirmFirstClickAt.value = null;
      }, 3000);
    }
  } else {
    if (props.clickSuccess) {
      showSuccess.value = true;
      successClickTimeout = setTimeout(() => {
        showSuccess.value = false;
      }, SHOW_SUCCESS_DELAY);
    }

    emit("click", e);
  }
}

const component = ref<InstanceType<typeof HTMLElement>>();
const focus = () => {
  component.value?.focus();
};

defineExpose({
  focus,
});

onBeforeUnmount(() => {
  if (successClickTimeout) clearTimeout(successClickTimeout);
});

const containerTheme = useTheme();

const computedTextSize = computed(() => {
  if (props.textSize) {
    return {
      "2xs": tw`text-2xs`,
      xs: tw`text-xs`,
      sm: tw`text-sm`,
      md: tw`text-md`,
      lg: tw`text-lg`,
      xl: tw`text-xl`,
    }[props.textSize];
  } else {
    return {
      "2xs": tw`text-[8px]`,
      xs: tw`text-[12px]`,
      sm: tw`text-[14px]`,
      md: tw`text-[14px]`,
      lg: tw`text-[18px]`,
      xl: tw`text-[20px]`,
    }[props.size];
  }
});

const computedClasses = computed(() => ({
  "--disabled": !!props.disabled,
  "--loading": !!computedLoading.value,
  ...(props.variant && { [`--variant-${props.variant}`]: true }),
  ...(props.size && { [`--size-${props.size}`]: true }),
  ...(props.textSize && { [`--text-size-${props.textSize}`]: true }),
  ...(props.tone && { [`--tone-${props.tone}`]: true }),
  "--rounded": !!props.rounded,
  "--hover-glow": !!props.hoverGlow,
  "--within-dark": containerTheme.theme.value === "dark",
  "--within-light": containerTheme.theme.value === "light",
  ...(props.square ? { "rounded-none": true } : { rounded: true }),
}));
</script>

<style lang="less">
.vbutton:focus-visible {
  outline: none;
}

.vbutton {
  display: inline-block;
  vertical-align: middle;
  border-style: solid;
  border-width: 1px;
  border-color: rgba(0, 0, 0, 0);
  // margin-right: 4px;
  // margin-bottom: 1px;
  // font-size: 14px;
  // font-family: @fancy-font;
  // text-transform: uppercase;
  text-decoration: none;
  font-weight: bold;
  text-align: center;
  text-shadow: none;
  white-space: nowrap;
  cursor: pointer;
  user-select: none;
  transition: all 0.25s;
  transition-property: color, border-color, background-color;
  position: relative;
  z-index: 2;
  // border-color: rgba(0,0,0,0) !important;
  // background-color: rgba(0,0,0,0) !important;

  > * {
    pointer-events: none;
  }

  .vbutton__inner {
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    z-index: 2;
    gap: 8px;
  }
  .vbutton__text {
    line-height: 24px;
    &:empty {
      display: none;
    }
  }
  .vbutton__icon {
    flex-grow: 0;
    flex-shrink: 0;
    pointer-events: none;
  }

  // &:focus {
  // }
  // &:hover {
  // }

  // Size options (medium is default)
  &.--size-2xs {
    // font-size: 8px;
    padding: 1px 1px;
    // border-radius: 8px;
    .vbutton__icon {
      padding: 0px;
    }
    .vbutton__inner {
      gap: 1px;
      padding: 0 0px;
    }
    .vbutton__text {
      padding: 0 1px;
    }
  }
  &.--size-xs {
    // font-size: 12px;
    padding: 2px 2px;
    // border-radius: 8px;
    .vbutton__icon {
      padding: 0px;
    }
    .vbutton__inner {
      gap: 2px;
      padding: 0 0px;
    }
    .vbutton__text {
      padding: 0 2px;
    }
  }
  &.--size-sm {
    // font-size: 14px;
    padding: 2px 4px;
    .vbutton__inner {
      gap: 2px;
    }
    .vbutton__icon {
      padding: 4px;
    }
  }

  &.--size-md {
    padding: 6px 8px;
    .vbutton__inner {
      gap: 4px;
    }
    .vbutton__icon {
      width: 24px;
      height: 24px;
    }
  }

  &.--size-lg {
    padding: 14px 24px;
    border-width: 2px;
    // font-size: 18px;
    .vbutton__inner {
      gap: 8px;
    }
    .vbutton__icon {
      width: 32px;
      height: 32px;
      margin-top: -4px; // to keep the icon size 24x24
      margin-bottom: -4px;
    }
  }

  &.--size-xl {
    max-width: 100%;
    // font-size: 20px;
    padding: 22px 36px;
    border-width: 2px;
    .vbutton__inner {
      gap: 16px;
    }
    .vbutton__icon {
      width: 40px;
      height: 40px;
      margin-top: -8px; // to keep the icon size 24x24
      margin-bottom: -8px;
    }
  }

  &.--disabled,
  &.--loading {
    pointer-events: none;
    opacity: 0.4;
    // cannot change cursor since pointer events are disabled
  }

  &:active {
    transform: scale3d(0.95, 0.95, 1);
  }

  // Set up theme helpers so we can quickly add new color themes
  .button-theme-generator(@color) {
    --button-glow-color: @color;
    --button-glow-color-darker: darken(@color, 20%);

    &.--variant-solid {
      background-color: @color;

      color: if(
        @color = @colors-action-300,
        // an exception to the rule for action 300
        "white",
        // sets legible text color based on the background color
        contrast(@color)
      );

      // set hover to either lighten or darken depending on if color is bright or dark
      &:hover when (lightness(@color) > 50%) {
        background: darken(@color, 10%);
      }
      &:hover when (lightness(@color) < 50%) {
        background: lighten(@color, 10%);
      }
    }
    &.--variant-ghost {
      color: @color;
      border-color: @color;
      background-color: fade(@color, 0);
      &:hover {
        background-color: fade(@color, 15);
      }
      &:active {
        background-color: fade(@color, 20);
      }
    }
    &.--variant-soft {
      color: @color;
      background: fade(@color, 15);
      &:hover {
        background: fade(@color, 20);
      }
      &:active {
        background: fade(@color, 25);
      }
    }
    &.--variant-transparent {
      color: @color;
      background: fade(@color, 0);
      &:hover {
        background: fade(@color, 15);
      }
      &:active {
        background: fade(@color, 25);
      }
    }
  }
  .create-transparent-button-theme(@color, @alpha: 0) {
    border-color: @color;
    background: fade(@color, @alpha);
    color: @color !important;

    &:hover {
      background: fade(@color, 10);
      border-color: @color;
    }
  }

  &.--tone-action {
    &.--within-dark {
      .button-theme-generator(@colors-action-300);
    }
    &.--within-light {
      .button-theme-generator(@colors-action-500);
    }
  }
  &.--tone-destructive {
    .button-theme-generator(@colors-destructive-500);
  }
  &.--tone-success {
    .button-theme-generator(@colors-success-500);
  }
  &.--tone-warning {
    .button-theme-generator(@colors-warning-500);
  }
  &.--tone-neutral {
    .button-theme-generator(@colors-neutral-500);
  }
  &.--tone-shade {
    &.--within-dark {
      .button-theme-generator(@colors-white);
    }
    &.--within-light {
      .button-theme-generator(@colors-black);
    }
  }

  &.--rounded {
    border-radius: 9999px;
  }

  &.--hover-glow:hover {
    animation: 1s glow_start, 1s glow 1s ease-in-out infinite alternate;
  }

  @keyframes glow_start {
    from {
      box-shadow: none;
    }

    to {
      box-shadow: 0 0 2px 1px rgba(0, 0, 0, 60%),
        0 0 15px var(--button-glow-color), 0 0 25px var(--button-glow-color),
        0 0 35px var(--button-glow-color), 0 0 45px var(--button-glow-color),
        0 0 55px var(--button-glow-color);
    }
  }

  @keyframes glow {
    from {
      box-shadow: 0 0 2px 1px rgba(0, 0, 0, 60%),
        0 0 15px var(--button-glow-color), 0 0 25px var(--button-glow-color),
        0 0 35px var(--button-glow-color), 0 0 45px var(--button-glow-color),
        0 0 55px var(--button-glow-color);
    }

    to {
      box-shadow: 0 0 2px 1px rgba(0, 0, 0, 60%),
        0 0 5px var(--button-glow-color), 0 0 10px var(--button-glow-color),
        0 0 20px var(--button-glow-color),
        0 0 30px var(--button-glow-color-darker),
        0 0 40px var(--button-glow-color-darker);
    }
  }
}
</style>
