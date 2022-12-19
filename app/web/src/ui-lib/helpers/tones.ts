import { tw } from "@/utils/style_helpers";

const TONES = {
  action: {
    bgColorClass: tw`bg-action-500`,
    textColorClass: tw`text-action-500`,
  },
  info: {
    bgColorClass: tw`bg-action-500`,
    textColorClass: tw`text-action-500`,
  },
  destructive: {
    bgColorClass: tw`bg-destructive-500`,
    textColorClass: tw`text-destructive-500`,
  },
  error: {
    bgColorClass: tw`bg-destructive-500`,
    textColorClass: tw`text-destructive-500`,
  },
  success: {
    bgColorClass: tw`bg-success-500`,
    textColorClass: tw`text-success-500`,
  },
  warning: {
    bgColorClass: tw`bg-warning-500`,
    textColorClass: tw`text-warning-500`,
  },
  neutral: {
    bgColorClass: tw`bg-neutral-500`,
    textColorClass: tw`text-neutral-500`,
  },
  // maybe swap dark/light?
  shade: { bgColorClass: tw`bg-shade-0`, textColorClass: tw`text-shade-0` },
};

export type Tones = keyof typeof TONES;

export function getToneBgColorClass(tone: Tones) {
  return TONES[tone].bgColorClass;
}

export function getToneTextColorClass(tone: Tones) {
  return TONES[tone].textColorClass;
}
