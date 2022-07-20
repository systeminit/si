<template>
  <button
    type="button"
    class="inline-flex items-center justify-center border rounded-[0.1875rem] focus:outline-none focus:ring-2 focus:ring-offset-2 font-sans font-semibold"
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
    <InputLabel v-if="iconStyle != 'alone'" :class="labelClasses">
      {{ label }}
    </InputLabel>
    <Icon
      v-if="icon && displayRightIcon"
      :icon="icon"
      :icon-classes="rightIconClasses"
    />
  </button>
</template>

<script setup lang="ts">
import { computed } from "vue";
import InputLabel from "@/atoms/InputLabel.vue";
import Icon from "./Button/Icon.vue";
import {
  ButtonColor,
  ButtonIcon,
  ButtonIconStyle,
  ButtonSize,
  ButtonStyle,
} from "./Button/types";

type IconPosition = "left" | "right" | "alone";
// type ButtonState = "normal" | "hover" | "focus" | "disable";

const emit = defineEmits(["click"]);

const props = withDefaults(
  defineProps<{
    label: string;
    toolTip?: string;
    color?: ButtonColor;
    size?: ButtonSize;
    buttonStyle?: ButtonStyle;
    iconStyle?: ButtonIconStyle;
    icon?: ButtonIcon;
    disabled?: boolean;
  }>(),
  {
    toolTip: undefined,
    color: "neutral",
    size: "md",
    buttonStyle: "primary",
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
  const a = sizeClasses[props.size];
  const b = styleClasses[props.buttonStyle];
  const c = colorClasses[props.color][props.buttonStyle];
  return a.concat(b).concat(c);
});

const labelClasses = computed(() => {
  return labelSizeClasses[props.size];
});

const leftIconClasses = computed(() => {
  return iconSizeClasses[props.size].left;
});

const rightIconClasses = computed(() => {
  return iconSizeClasses[props.size].right;
});

const aloneIconClasses = computed(() => {
  return iconSizeClasses[props.size].alone;
});

const sizeClasses: { [key in ButtonSize]: string[] } = {
  lg: ["px-[0.3125rem]", "py-2.5", "text-xs"],
  md: ["px-[0.3125rem]", "py-2", "text-xs"],
  sm: ["px-[0.3125rem]", "py-1.5", "text-xs"],
  xs: ["px-[0.3125rem]", "py-1", "text-xs"],
};

const styleClasses: { [key in ButtonStyle]: string[] } = {
  primary: ["border-transparent", "shadow-sm", "text-white"],
  secondary: ["border-transparent"],
  outlined: [],
  link: ["border-transparent"],
};

const colorClasses: {
  [key in ButtonColor]: { [key in ButtonStyle]: string[] };
} = {
  neutral: {
    primary: [
      "bg-neutral-500",
      "hover:bg-neutral-600",
      "focus:ring-neutral-400",
    ],
    secondary: [
      "text-neutral-500",
      "bg-neutral-50",
      "hover:bg-neutral-100",
      "focus:ring-neutral-400",
    ],
    outlined: [
      "border-neutral-300",
      "text-neutral-500",
      "hover:bg-neutral-50",
      "focus:ring-neutral-400",
    ],
    link: [
      "text-neutral-500",
      "hover:text-neutral-600",
      "focus:ring-neutral-400",
    ],
  },
  action: {
    primary: ["bg-action-500", "hover:bg-action-600", "focus:ring-action-400"],
    secondary: [
      "text-action-500",
      "bg-action-50",
      "hover:bg-action-100",
      "focus:ring-action-400",
    ],
    outlined: [
      "border-action-300",
      "text-action-500",
      "hover:bg-action-50",
      "focus:ring-action-400",
    ],
    link: ["text-action-500", "hover:text-action-700", "focus:ring-action-400"],
  },
  success: {
    primary: [
      "bg-success-500",
      "hover:bg-success-600",
      "focus:ring-success-400",
    ],
    secondary: [
      "text-success-500",
      "bg-success-50",
      "hover:bg-success-100",
      "focus:ring-success-400",
    ],
    outlined: [
      "border-success-300",
      "text-success-500",
      "hover:bg-success-50",
      "focus:ring-success-400",
    ],
    link: [
      "text-success-500",
      "hover:text-success-600",
      "focus:ring-success-400",
    ],
  },
  warning: {
    primary: [
      "bg-warning-50",
      "hover:bg-warning-600",
      "focus:ring-warning-400",
    ],
    secondary: [
      "text-warning-500",
      "bg-warning-50",
      "hover:bg-warning-100",
      "focus:ring-warning-400",
    ],
    outlined: [
      "border-warning-300",
      "text-warning-500",
      "hover:bg-warning-50",
      "focus:ring-warning-400",
    ],
    link: [
      "text-warning-500",
      "hover:text-warning-600",
      "focus:ring-warning-400",
    ],
  },
  destructive: {
    primary: [
      "bg-destructive-500",
      "hover:bg-destructive-700",
      "focus:ring-destructive-500",
    ],
    secondary: [
      "text-destructive-500",
      "bg-destructive-50",
      "hover:bg-destructive-100",
      "focus:ring-destructive-400",
    ],
    outlined: [
      "border-destructive-300",
      "text-destructive-500",
      "hover:bg-destructive-50",
      "focus:ring-destructive-400",
    ],
    link: [
      "text-destructive-500",
      "hover:text-destructive-600",
      "focus:ring-destructive-400",
    ],
  },
};

const labelSizeClasses: { [key in ButtonSize]: string[] } = {
  lg: [],
  md: [],
  sm: [],
  xs: [],
};

const iconSizeClasses: {
  [key in ButtonSize]: { [key in IconPosition]: string[] };
} = {
  lg: {
    left: ["mr-[0.1875rem]", "h-5", "w-5"],
    right: ["ml-[0.1875rem]", "h-5", "w-5"],
    alone: ["mx-[0.3125rem]", "h-5", "w-5"],
  },
  md: {
    left: ["mr-[0.1875rem]", "h-5", "w-5"],
    right: ["ml-[0.1875rem]", "h-5", "w-5"],
    alone: ["mx-[0.3125rem]", "h-5", "w-5"],
  },
  sm: {
    left: ["mr-[0.1875rem]", "h-5", "w-5"],
    right: ["ml-[0.1875rem]", "h-5", "w-5"],
    alone: ["mx-[0.3125rem]", "h-5", "w-5"],
  },
  xs: {
    left: ["mr-[0.1875rem]", "h-5", "w-5"],
    right: ["ml-[0.1875rem]", "h-5", "w-5"],
    alone: ["mx-[0.3125rem]", "h-5", "w-5"],
  },
};
</script>
