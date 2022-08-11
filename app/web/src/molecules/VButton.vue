<template>
  <button
    type="button"
    class="inline-flex items-center justify-center rounded-[0.1875rem] border text-sm focus:outline-none focus:ring-1 focus:ring-action-500 focus:ring-offset-2 disabled:opacity-50"
    :class="buttonClasses"
    :aria-label="label"
    :disabled="disabled"
    @click="emit('click')"
  >
    <Icon
      v-if="icon && displayLeftIcon"
      :icon="icon"
      :icon-classes="leftIconClasses"
    />
    <Icon
      v-if="icon && displayAloneIcon"
      :icon="icon"
      :icon-classes="aloneIconClasses"
    />
    <div v-if="iconStyle != 'alone'">
      {{ label }}
    </div>
    <Icon
      v-if="icon && displayRightIcon"
      :icon="icon"
      :icon-classes="rightIconClasses"
    />
  </button>
</template>

<script setup lang="ts">
import { computed } from "vue";
import Icon, { IconName } from "./VButton/Icon.vue";

export type ButtonType =
  | "neutral"
  | "action"
  | "success"
  | "warning"
  | "destructive";

export type ButtonSize = "lg" | "md" | "sm" | "xs";

export type ButtonRank = "primary" | "secondary" | "tertiary";

export type ButtonIconStyle =
  | "leftAndRight"
  | "left"
  | "right"
  | "none"
  | "alone";

type IconPosition = "left" | "right" | "alone";

const emit = defineEmits(["click"]);

const props = withDefaults(
  defineProps<{
    size?: ButtonSize;
    buttonRank?: ButtonRank;
    buttonType?: ButtonType;
    label: string;
    toolTip?: string;

    disabled?: boolean;

    iconStyle?: ButtonIconStyle;
    icon?: IconName;
  }>(),
  {
    size: "md",
    buttonRank: "primary",
    buttonType: "neutral",

    toolTip: undefined,
    iconStyle: "none",
    icon: undefined,
  },
);

const displayLeftIcon = computed(() => {
  return props.iconStyle == "left" || props.iconStyle == "leftAndRight";
});

const displayRightIcon = computed(() => {
  return props.iconStyle == "right" || props.iconStyle == "leftAndRight";
});

const displayAloneIcon = computed(() => {
  return props.iconStyle == "alone";
});

const buttonClasses = computed(() => {
  return [
    ...sizeClasses[props.size],
    ...buttonRankClasses[props.buttonRank],
    ...buttonTypeClasses[props.buttonType][props.buttonRank],
  ];
});

const leftIconClasses = computed(() => {
  return buttonIconClasses.left;
});

const rightIconClasses = computed(() => {
  return buttonIconClasses.right;
});

const aloneIconClasses = computed(() => {
  return buttonIconClasses.alone;
});

const sizeClasses: { [key in ButtonSize]: string[] } = {
  lg: ["px-[0.3125rem]", "py-[0.4375rem]"],
  md: ["px-[0.3125rem]", "py-[0.4375rem]"],
  sm: ["px-[0.3125rem]", "py-[0.4375rem]"],
  xs: ["px-[0.3125rem]", "py-[0.4375rem]"],
};

const buttonRankClasses: { [key in ButtonRank]: string[] } = {
  primary: ["border-transparent", "shadow-sm", "text-white"],
  secondary: [],
  tertiary: ["border-transparent"],
};

const buttonTypeClasses: {
  [key in ButtonType]: { [key in ButtonRank]: string[] };
} = {
  neutral: {
    primary: ["bg-neutral-500", "hover:bg-neutral-600", "focus:bg-neutral-700"],
    secondary: [
      "text-neutral-500",
      "border-neutral-500",
      "hover:border-neutral-600",
      "hover:bg-neutral-50",
      "focus:border-neutral-700",
      "focus:bg-neutral-100",
    ],
    tertiary: [
      "text-neutral-500",
      "hover:bg-neutral-50",
      "focus:bg-neutral-100",
    ],
  },
  action: {
    primary: ["bg-action-500", "hover:bg-action-600", "focus:bg-action-700"],
    secondary: [
      "text-action-500",
      "border-action-500",
      "hover:border-action-600",
      "hover:bg-action-50",
      "focus:border-action-700",
      "focus:bg-action-100",
    ],
    tertiary: ["text-action-500", "hover:bg-action-50", "focus:bg-action-100"],
  },
  success: {
    primary: ["bg-success-500", "hover:bg-success-600", "focus:bg-success-700"],
    secondary: [
      "text-success-500",
      "border-success-500",
      "hover:border-success-600",
      "hover:bg-success-50",
      "focus:border-success-700",
      "focus:bg-success-100",
    ],
    tertiary: [
      "text-success-500",
      "hover:bg-success-50",
      "focus:bg-success-100",
    ],
  },
  warning: {
    primary: ["bg-warning-500", "hover:bg-warning-600", "focus:bg-warning-700"],
    secondary: [
      "text-warning-500",
      "border-warning-500",
      "hover:border-warning-600",
      "hover:bg-warning-50",
      "focus:border-warning-700",
      "focus:bg-warning-100",
    ],
    tertiary: [
      "text-warning-500",
      "hover:bg-warning-50",
      "focus:bg-warning-100",
    ],
  },
  destructive: {
    primary: [
      "bg-destructive-500",
      "hover:bg-destructive-600",
      "focus:bg-destructive-700",
    ],
    secondary: [
      "text-destructive-500",
      "border-destructive-500",
      "hover:border-destructive-600",
      "hover:bg-destructive-50",
      "focus:border-destructive-700",
      "focus:bg-destructive-100",
    ],
    tertiary: [
      "text-destructive-500",
      "hover:bg-destructive-50",
      "focus:bg-destructive-100",
    ],
  },
};

const buttonIconClasses: { [key in IconPosition]: string[] } = {
  left: ["mr-[0.1875rem]", "h-5", "w-5"],
  right: ["ml-[0.1875rem]", "h-5", "w-5"],
  alone: ["mx-[0.3125rem]", "h-5", "w-5"],
};
</script>
