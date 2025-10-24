import { tw } from "../../utils/tw-utils";

import colorsJson from "../../tailwind/tailwind_customization/colors.json";
import { useTheme } from "./theme_tools";

export const COLOR_PALETTE = colorsJson;

export const BRAND_COLOR_FILTER_HEX_CODES = {
  "Digital Ocean": "#ABF7FF",
  "Google Cloud": "#EF6255",
  "Open AI": "#FFBE19",
  AWS: "#FF9900",
  Azure: "#104581",
  CoreOS: "#E9659C",
  Custom: "#E5E5E5",
  Docker: "#1D63ED",
  Fastly: "#FE272D",
  GitHub: "#9467EC",
  Helpers: "#E1A7FE",
  Hetzner: "#D50C2D",
  Netlify: "#05BDBA",
  Posthog: "#F54E00",
  Tailscale: "#A7DAC2",
};

// TODO(Wendy) - a LOT of this system is outdated, we need to revamp it!
const TONES = {
  action: {
    colorHexLight: COLOR_PALETTE.action[500],
    colorHexDark: COLOR_PALETTE.action[300],
    bgColorClassLight: tw`bg-action-500`,
    bgColorClassDark: tw`bg-action-300`,
    textColorClassLight: tw`text-action-500`,
    textColorClassDark: tw`text-action-300`,
    borderColorClassLight: tw`border-action-500`,
    borderColorClassDark: tw`border-action-300`,
    iconColorClassLight: tw`text-action-500`,
    iconColorClassDark: tw`text-action-300`,
  },
  info: {
    colorHexLight: COLOR_PALETTE.action[500],
    colorHexDark: COLOR_PALETTE.action[300],
    bgColorClass: tw`bg-action-500`,
    textColorClassLight: tw`text-action-500`,
    textColorClassDark: tw`text-action-300`,
    borderColorClassLight: tw`border-action-500`,
    borderColorClassDark: tw`border-action-300`,
    iconColorClassLight: tw`text-action-500`,
    iconColorClassDark: tw`text-action-300`,
  },
  destructive: {
    colorHexLight: COLOR_PALETTE.destructive[600],
    colorHexDark: COLOR_PALETTE.destructive[200],
    bgColorClassLight: tw`bg-destructive-500`,
    bgColorClassDark: tw`bg-destructive-600`,
    textColorClassLight: tw`text-destructive-500`,
    textColorClassDark: tw`text-destructive-600`,
    borderColorClassLight: tw`border-destructive-500`,
    borderColorClassDark: tw`border-destructive-600`,
    iconColorClassLight: tw`text-destructive-600`,
    iconColorClassDark: tw`text-destructive-300`,
  },
  error: {
    colorHexLight: COLOR_PALETTE.destructive[600],
    colorHexDark: COLOR_PALETTE.destructive[200],
    bgColorClass: tw`bg-destructive-500`,
    textColorClass: tw`text-destructive-500`,
    borderColorClass: tw`border-destructive-500`,
    iconColorClassLight: tw`text-destructive-600`,
    iconColorClassDark: tw`text-destructive-300`,
  },
  success: {
    colorHexLight: COLOR_PALETTE.success[600],
    colorHexDark: COLOR_PALETTE.success[300],
    bgColorClassLight: tw`bg-success-500`,
    bgColorClassDark: tw`bg-success-300`,
    textColorClass: tw`text-success-500`,
    borderColorClass: tw`border-success-500`,
    iconColorClassLight: tw`text-success-600`,
    iconColorClassDark: tw`text-success-300`,
  },
  warning: {
    colorHexLight: COLOR_PALETTE.warning[500],
    colorHexDark: COLOR_PALETTE.warning[200],
    bgColorClassLight: tw`bg-warning-500`,
    bgColorClassDark: tw`bg-warning-400`,
    textColorClassLight: tw`text-warning-500`,
    textColorClassDark: tw`text-warning-400`,
    borderColorClassLight: tw`border-warning-500`,
    borderColorClassDark: tw`border-warning-400`,
    iconColorClassLight: tw`text-warning-500`,
    iconColorClassDark: tw`text-warning-200`,
  },
  neutral: {
    colorHex: COLOR_PALETTE.neutral[500],
    bgColorClass: tw`bg-neutral-500`,
    textColorClass: tw`text-neutral-500`,
    borderColorClass: tw`border-neutral-500`,
    iconColorClass: tw`text-neutral-500`,
  },
  empty: {
    colorHex: "#00000000",
    bgColorClass: tw`bg-transparent`,
    textColorClass: tw`text-transparent border-transparent bg-transparent border-0 border-hidden opacity-0`,
    borderColorClass: tw`border-transparent`,
    iconColorClass: tw`text-transparent border-transparent bg-transparent border-0 border-hidden opacity-0`,
  },
  shade: {
    colorHexLight: COLOR_PALETTE.shade[0],
    colorHexDark: COLOR_PALETTE.shade[100],
    bgColorClassLight: tw`bg-shade-0`,
    bgColorClassDark: tw`bg-shade-100`,
    textColorClassLight: tw`text-shade-100`,
    textColorClassDark: tw`text-shade-0`,
    borderColorClassLight: tw`bg-shade-0`,
    borderColorClassDark: tw`bg-shade-100`,
    iconColorClassLight: tw`text-shade-100`,
    iconColorClassDark: tw`text-shade-0`,
  },
};

export type Tones = keyof typeof TONES;

export const ColorNamesArray = [
  "neutral",
  "action",
  "success",
  "warning",
  "destructive",
  "shade",
] as const;

export type ColorNames = (typeof ColorNamesArray)[number];

export function getToneBgColorClass(tone: Tones) {
  const { theme } = useTheme();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const toneSettings = TONES[tone] as any;
  if (theme.value === "dark")
    return toneSettings.bgColorClassDark || toneSettings.bgColorClass;
  return toneSettings.bgColorClassLight || toneSettings.bgColorClass;
}

export function getToneTextColorClass(tone: Tones) {
  const { theme } = useTheme();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const toneSettings = TONES[tone] as any;

  if (theme?.value === "dark") {
    return toneSettings?.textColorClassDark ?? toneSettings.textColorClass;
  }

  return toneSettings?.textColorClassLight ?? toneSettings.textColorClass;
}

export function getToneBorderColorClass(tone: Tones) {
  const { theme } = useTheme();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const toneSettings = TONES[tone] as any;
  if (theme.value === "dark")
    return toneSettings.borderColorClassDark || toneSettings.borderColorClass;
  return toneSettings.borderColorClassLight || toneSettings.borderColorClass;
}

export function getToneColorHex(tone: Tones) {
  const { theme } = useTheme();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const toneSettings = TONES[tone] as any;
  if (theme.value === "dark")
    return toneSettings.colorHexDark || toneSettings.colorHex;
  return toneSettings.colorHexLight || toneSettings.colorHex;
}

export function getIconToneColorClass(tone: Tones) {
  const { theme } = useTheme();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const toneSettings = TONES[tone] as any;

  if (theme?.value === "dark") {
    return toneSettings?.iconColorClassDark ?? toneSettings.iconColorClass;
  }

  return toneSettings?.iconColorClassLight ?? toneSettings.iconColorClass;
}
