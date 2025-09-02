<!--
  New button component for the newhotness UI
  Do not use VButton anymore!
-->

<template>
  <component
    :is="htmlTagOrComponentType"
    v-bind="dynamicAttrs"
    ref="component"
    v-tooltip="truncateRef?.tooltipActive ? truncateRef.tooltip : undefined"
    :tabindex="tabIndex"
    :class="
      clsx(
        'flex flex-row items-center gap-xs transition-all justify-center px-xs py-2xs whitespace-nowrap leading-none font-medium rounded-sm',
        tone !== 'empty' && 'border',
        computedTextSize,
        truncateText && 'min-w-0',
        disabled
          ? [
              'cursor-not-allowed',
              themeClasses(
                'text-neutral-500 bg-neutral-200 border-neutral-300',
                'text-neutral-400 bg-neutral-800 border-neutral-700',
              ),
            ]
          : [
              'cursor-pointer hover:scale-105 active:scale-100',
              {
                neutral: themeClasses(
                  'text-neutral-900 bg-neutral-200 border-neutral-400 hover:bg-neutral-100',
                  'text-white bg-neutral-700 border-neutral-600 hover:bg-neutral-600',
                ),
                action: [
                  'text-white',
                  themeClasses(
                    'bg-[#1264BF] border-[#318AED] hover:bg-[#2583EC]',
                    'bg-[#1264BF] border-[#318AED] hover:bg-[#2583EC]',
                  ),
                ],
                warning: themeClasses(
                  'text-neutral-900 bg-[#F4F0EC] border-warning-500 hover:bg-white',
                  'text-white bg-[#432D1D] border-[#98511B] hover:bg-[#67452D]',
                ),
                destructive: themeClasses(
                  'text-neutral-900 bg-destructive-50 border-destructive-300 hover:bg-white',
                  'text-white bg-[#341C1C] border-[#A93232] hover:bg-[#562E2E]',
                ),
                empty: '',
              }[tone],
            ],
      )
    "
    @click="clickHandler($event)"
  >
    <template v-if="computedLoading">
      <Icon
        :size="iconSize ? iconSize : size"
        :class="iconClasses"
        :name="loadingIcon"
      />
      <span>{{ loadingText }}</span>
    </template>
    <template v-else-if="showSuccess">
      <Icon
        :size="iconSize ? iconSize : size"
        :class="iconClasses"
        :name="successIcon"
      />
      <span>
        <slot name="success">{{ successText }}</slot>
      </span>
    </template>
    <template v-else>
      <slot name="icon">
        <Icon
          v-if="icon"
          :size="iconSize ? iconSize : size"
          :class="iconClasses"
          :name="icon"
        />
      </slot>
      <TruncateWithTooltip v-if="truncateText" ref="truncateRef">
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
      <span v-else-if="label || $slots.default">
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
        <Icon
          v-if="iconRight"
          :size="iconSize ? iconSize : size"
          :class="iconClasses"
          :name="iconRight"
        />
      </slot>
      <slot name="pill">
        <TextPill
          v-if="pill"
          :class="
            clsx(
              'ml-2xs min-w-[22px] text-center',
              {
                neutral: '',
                action: '',
                warning: '',
                destructive: '',
                empty: '',
              }[props.tone],
            )
          "
          mono
        >
          {{ pill }}
        </TextPill>
      </slot>
    </template>
  </component>
</template>

<script lang="ts" setup>
import { ref, computed, onBeforeUnmount, watch, PropType } from "vue";
import { RouterLink } from "vue-router";
import * as _ from "lodash-es";
import clsx from "clsx";
import { ApiRequestStatus } from "../../pinia";
import Icon, { IconSizes } from "../icons/Icon.vue";
import { IconNames } from "../icons/icon_set";
import TextPill from "./TextPill.vue";
import TruncateWithTooltip from "./TruncateWithTooltip.vue";
import { tw } from "../../utils/tw-utils";
import { themeClasses } from "../utils/theme_tools";

const SHOW_SUCCESS_DELAY = 2000;

export type ButtonSizes = "2xs" | "xs" | "sm" | "md" | "lg" | "xl";
export type ButtonTones =
  | "neutral"
  | "action"
  | "warning"
  | "destructive"
  | "empty";

const props = defineProps({
  size: { type: String as PropType<ButtonSizes>, default: "sm" },
  iconSize: { type: String as PropType<IconSizes> },
  iconClasses: { type: String, default: tw`flex-none pointer-events-none` },
  textSize: { type: String as PropType<ButtonSizes> },
  tone: { type: String as PropType<ButtonTones>, default: "neutral" },
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
  pill: { type: String, required: false },
  truncateText: { type: Boolean },
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
  if (props.disabled) {
    e.stopPropagation();
    e.preventDefault();
    return;
  }

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
</script>
