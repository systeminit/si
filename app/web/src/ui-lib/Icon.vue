<!--
  General Icon component to use throughout the codebase

  Why not just import the icons directly?
  - single import rather than importing many icons in each file, no need to change import to try different icon
  - easier to keep icons consistent and swap all icons of a certain type at once (ex: use the same "x-circle" everywhere)
  - allows multiple aliases for the same icon so the use can be a bit more specific (ex: "qualification-passing")
  - easier to apply consistent styling throughout
  - using a simple string lets us easily add `icon` properties on other components (like buttons / form inputs)
  - rotation helpers so we can use a single icon for each direction of things like arrows / carets
-->

<template>
  <div
    class="icon"
    :class="
      clsx(
        'block pointer-events-none transition-transform duration-300',
        sizeClasses,
      )
    "
  >
    <component
      :is="iconComponent"
      class="icon__svg"
      :class="
        clsx(svgRotateClass, 'w-full h-full', {
          'animate-spin': AUTO_SPIN_ICONS.includes(props.name),
        })
      "
    />
  </div>
</template>

<script lang="ts">
/* eslint-disable import/no-unresolved,import/extensions,vue/component-tags-order,import/first,import/order */

// browse available icons at https://icones.js.org/ (or https://iconify.design/icon-sets/)

import Loader from "~icons/gg/spinner";
import Check from "~icons/material-symbols/check";
import CheckCircle from "~icons/material-symbols/check-circle";
import AlertTriangle from "~icons/material-symbols/warning";
import XCircle from "~icons/material-symbols/cancel";
import QuestionMarkCircle from "~icons/heroicons-solid/question-mark-circle";
import Play from "~icons/material-symbols/play-arrow-rounded";

import PlusSquare from "~icons/fa/plus-square-o";

import Arrow from "~icons/heroicons-solid/arrow-up";
import Chevron from "~icons/heroicons-solid/chevron-up";

import Minus from "~icons/heroicons-solid/minus";
import MinusCircle from "~icons/heroicons-solid/minus-circle";
import Plus from "~icons/heroicons-solid/plus";
import PlusCircle from "~icons/heroicons-solid/plus-circle";
import Save from "~icons/heroicons-solid/save";
import Trash from "~icons/heroicons-solid/trash";
import X from "~icons/heroicons-solid/x";
import PlayCircle from "~icons/heroicons-solid/play";
import Beaker from "~icons/heroicons-solid/beaker";
import Link from "~icons/heroicons-solid/link";
import Moon from "~icons/heroicons-solid/moon";
import Sun from "~icons/heroicons-solid/sun";
import Eye from "~icons/heroicons-solid/eye";
import ClipboardCopy from "~icons/heroicons-solid/clipboard-copy";
import Refresh from "~icons/heroicons-solid/refresh";
import Pencil from "~icons/heroicons-outline/pencil";
import Cube from "~icons/heroicons-outline/cube";
import Clock from "~icons/heroicons-solid/clock";
import ExclamationCircle from "~icons/heroicons-solid/exclamation-circle";
import CreditCard from "~icons/heroicons-solid/credit-card";
import Bell from "~icons/heroicons-solid/bell";
import CheckBadge from "~icons/heroicons-solid/badge-check";
import DotsHorizontal from "~icons/heroicons-solid/dots-horizontal";
import DotsVertical from "~icons/heroicons-solid/dots-vertical";
import Search from "~icons/heroicons-solid/search";
import Selector from "~icons/heroicons-solid/selector";

// octicons (from github) available as no suffix, -16, -24
import GitBranch from "~icons/octicon/git-branch-24";
import GitCommit from "~icons/octicon/git-commit-24";
import GitMerge from "~icons/octicon/git-merge-24";

// custom icons
import Diagram from "@/assets/images/custom-icons/diagram.svg?component";

// restricting the type here (Record<string, FunctionalComponent>) kills our IconName type below
/* eslint sort-keys: "error" */
const ICON_NAME_MAP = Object.freeze({
  "alert-triangle": AlertTriangle,
  beaker: Beaker,
  bell: Bell,
  check: Check,
  "check-badge": CheckBadge,
  "check-circle": CheckCircle,
  "clipboard-copy": ClipboardCopy,
  clock: Clock,
  component: Cube,
  "credit-card": CreditCard,
  diagram: Diagram,
  "dots-horizontal": DotsHorizontal,
  "dots-vertical": DotsVertical,
  edit: Pencil,
  "exclamation-circle": ExclamationCircle,
  eye: Eye,
  "git-branch": GitBranch,
  "git-commit": GitCommit,
  "git-merge": GitMerge,
  "help-circle": QuestionMarkCircle,
  link: Link,
  loader: Loader,
  minus: Minus,
  "minus-circle": MinusCircle,
  moon: Moon,
  play: Play,
  "play-circle": PlayCircle,
  plus: Plus,
  "plus-circle": PlusCircle,
  "plus-square": PlusSquare,
  refresh: Refresh,
  "refresh-active": Refresh,
  save: Save,
  search: Search,
  selector: Selector,
  sun: Sun,
  trash: Trash,
  x: X,
  "x-circle": XCircle,
});
/* eslint-disable sort-keys */

/*
  additional aliases which makes it easy to be more consistent with icon usage
  while still allowing us to change icons for specific cases later
*/
const ICON_NAME_ALIASES = {
  "qualification-passed": "check-circle",
  "qualification-failed": "x-circle",
};

// these icons are intended to be used with a specific direction, ex: "arrow--down"
// make sure the base icon is pointing up!
const SPINNABLE_ICON_NAME_MAP = Object.freeze({
  arrow: Arrow,
  // triangle: Triangle,
  chevron: Chevron,
});

type RegularIconNames = keyof typeof ICON_NAME_MAP;
type IconNameAliases = keyof typeof ICON_NAME_ALIASES;
type SpinnableRawIconNames = keyof typeof SPINNABLE_ICON_NAME_MAP;
type SpinnableIconNames = `${SpinnableRawIconNames}--${
  | "left"
  | "right"
  | "up"
  | "down"}`;

export type IconNames = RegularIconNames | IconNameAliases | SpinnableIconNames;
</script>

<script lang="ts" setup>
import { computed, FunctionalComponent, PropType } from "vue";
import clsx from "clsx";

const props = defineProps({
  name: { type: String as PropType<IconNames>, required: true },
  rotate: { type: String as PropType<"left" | "right" | "up" | "down"> },
  size: {
    type: String as PropType<"xs" | "s" | "m" | "l" | "xl" | "full">,
    default: "m",
  },
});

const iconComponent = computed(() => {
  const nameWithoutModifiers = props.name.split("--")[0];

  if (SPINNABLE_ICON_NAME_MAP[nameWithoutModifiers as SpinnableRawIconNames]) {
    return SPINNABLE_ICON_NAME_MAP[
      nameWithoutModifiers as SpinnableRawIconNames
    ] as FunctionalComponent;
  }
  if (ICON_NAME_MAP[nameWithoutModifiers as RegularIconNames]) {
    return ICON_NAME_MAP[
      nameWithoutModifiers as RegularIconNames
    ] as FunctionalComponent;
  }
  if (ICON_NAME_ALIASES[nameWithoutModifiers as IconNameAliases]) {
    return ICON_NAME_MAP[
      ICON_NAME_ALIASES[
        nameWithoutModifiers as IconNameAliases
      ] as RegularIconNames
    ] as FunctionalComponent;
  }

  return ICON_NAME_MAP["help-circle"] as FunctionalComponent;
});

const svgRotateClass = computed(() => {
  let rotate = "";
  if (props.rotate) rotate = props.rotate;
  // eslint-disable-next-line prefer-destructuring
  else if (props.name.includes("--")) rotate = props.name.split("--")[1];
  if (!rotate) return "";

  return {
    up: "rotate-0",
    right: "rotate-90",
    down: "rotate-180",
    left: "-rotate-90",
  }[rotate];
});

const sizeClasses = computed(() => {
  return {
    full: "w-full h-full",
    xs: "w-4 h-4",
    s: "w-5 h-5",
    m: "w-6 h-6",
    l: "w-8 h-8",
    xl: "w-9 h-9",
  }[props.size];
});

const AUTO_SPIN_ICONS = ["loader", "refresh-active"];
</script>
