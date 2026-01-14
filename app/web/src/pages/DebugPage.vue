<template>
  <AppLayout pageMode="scroll">
    <meta name="robots" content="noindex" />
    <DebugNavbar />
    <div class="flex flex-col pt-sm pb-lg gap-sm items-center justify-center border-t border-white">
      <div class="w-full px-lg text-xl text-left">Main Icon Set</div>
      <div class="w-full flex flex-row flex-wrap gap-sm px-lg pb-lg justify-start">
        <Icon
          v-for="(_, name) in ICONS"
          :key="name"
          v-tooltip="name"
          :name="name as IconNames"
          class="cursor-pointer"
        />
        <template v-for="(_, name) in SPINNABLE_ICONS" :key="name">
          <Icon
            v-for="d in ['left', 'right', 'up', 'down']"
            :key="d"
            v-tooltip="`${name}--${d}`"
            :name="`${name}--${d}` as IconNames"
            class="cursor-pointer"
          />
        </template>
      </div>

      <div class="w-full px-lg text-xl text-left">Logo Icons</div>
      <div class="w-full flex flex-row flex-wrap gap-sm px-lg pb-lg justify-start">
        <Icon
          v-for="(_, name) in LOGO_ICONS"
          :key="name"
          v-tooltip="name"
          :name="name as IconNames"
          class="cursor-pointer"
        />
      </div>

      <div class="w-full px-lg text-xl text-left">Button Garden (NEW UI)</div>
      <div
        v-for="num in Array(6).keys()"
        :key="num"
        :class="clsx('w-full flex flex-row gap-sm flex-wrap px-lg justify-start h-8', num === 5 && 'mb-lg')"
      >
        <template v-for="variant in buttonVariants" :key="variant">
          <NewButton
            v-tooltip="getButtonTooltip(variant)"
            :label="variant.charAt(0).toUpperCase() + variant.slice(1)"
            :tone="variant !== 'disabled' ? (variant as ButtonTones) : undefined"
            :pill="num > 3 ? 'test' : undefined"
            :icon="num === 1 || num === 5 ? 'cat' : undefined"
            :iconRight="num === 2 ? 'cat' : undefined"
            :loading="num === 3"
            :disabled="variant === 'disabled'"
          />
        </template>
      </div>

      <div class="w-full px-lg text-xl text-left">Empty State Icons (OLD UI)</div>
      <div class="w-full flex flex-row flex-wrap px-lg pb-lg justify-start">
        <div v-for="(_, name) in BIG_ICONS" :key="name" class="basis-1/4">
          <EmptyStateIcon v-tooltip="name" :name="name" class="w-full cursor-pointer" />
        </div>
      </div>

      <div class="w-full px-lg text-xl text-left">Other SVGs</div>
      <div class="w-full flex flex-row flex-wrap px-lg pb-lg justify-start gap-sm items-center">
        <NodeSkeleton v-tooltip="'NodeSkeleton'" color="#ff00ff" class="cursor-pointer" />
        <SiLogo v-tooltip="'SiLogo'" class="cursor-pointer h-xl w-xl" />
        <CheechSvg
          v-tooltip="'CheechSvg'"
          class="h-xl w-xl rounded-full bg-shade-0 border-2 border-shade-100 cursor-pointer"
        />
      </div>

      <div class="w-full px-lg text-xl text-left">Semantic Sizes Reference</div>
      <div class="flex flex-col w-full gap-2xs">
        <div class="px-lg w-full flex flex-row justify-between font-bold gap-sm">
          <div class="basis-1/6 text-center">3xs: 0.125rem / 2px</div>
          <div class="basis-1/6 text-center">2xs: 0.25rem / 4px</div>
          <div class="basis-1/6 text-center">xs: 0.5rem / 8px</div>
          <div class="basis-1/6 text-center">sm: 1rem / 16px</div>
          <div class="basis-1/6 text-center">md: 1.5rem / 24px</div>
          <div class="basis-1/6 text-center">lg: 2.25rem / 36px</div>
          <div class="basis-1/6 text-center">xl: 4rem / 64px</div>
          <div class="basis-1/6 text-center">2xl: 6rem / 96px</div>
          <div class="basis-1/6 text-center">3xl: 8rem / 128px</div>
        </div>
        <div class="w-full flex flex-row px-lg justify-between gap-sm items-center">
          <div class="basis-[12.5%] flex flex-row items-start justify-center">
            <div class="w-3xs h-3xs bg-action-500"></div>
          </div>
          <div class="basis-[12.5%] flex flex-row items-start justify-center">
            <div class="w-2xs h-2xs bg-action-500"></div>
          </div>
          <div class="basis-[12.5%] flex flex-row items-center justify-center">
            <div class="w-xs h-xs bg-action-500"></div>
          </div>
          <div class="basis-[12.5%] flex flex-row items-center justify-center">
            <div class="w-sm h-sm bg-action-500"></div>
          </div>
          <div class="basis-[12.5%] flex flex-row items-center justify-center">
            <div class="w-md h-md bg-action-500"></div>
          </div>
          <div class="basis-[12.5%] flex flex-row items-center justify-center">
            <div class="w-lg h-lg bg-action-500"></div>
          </div>
          <div class="basis-[12.5%] flex flex-row items-center justify-center">
            <div class="w-xl h-xl bg-action-500"></div>
          </div>
          <div class="basis-[12.5%] flex flex-row items-center justify-center">
            <div class="w-2xl h-2xl bg-action-500"></div>
          </div>
          <div class="basis-[12.5%] flex flex-row items-start justify-center">
            <div class="w-3xl h-3xl bg-action-500"></div>
          </div>
        </div>
        <div class="px-lg w-full flex flex-row justify-between gap-sm">
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-3xs">3xs text</div>
            <div class="text-md">0.5rem / 8px</div>
          </div>
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-2xs">2xs text</div>
            <div class="text-md">0.6rem / 9.6px</div>
          </div>
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-xs">xs text</div>
            <div class="text-md">0.75rem / 12px</div>
          </div>
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-sm">sm text</div>
            <div class="text-md">0.875rem / 14px</div>
          </div>
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-md">md text</div>
            <div class="text-md">1rem / 16px</div>
          </div>
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-lg">lg text</div>
            <div class="text-md">1.125rem / 18px</div>
          </div>
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-xl">xl text</div>
            <div class="text-md">1.25rem / 20px</div>
          </div>
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-2xl">2xl text</div>
            <div class="text-md">1.5rem / 24px</div>
          </div>
          <div class="basis-[12.5%] text-center flex flex-col items-center justify-center">
            <div class="text-3xl">3xl text</div>
            <div class="text-md">1.875rem / 30px</div>
          </div>
        </div>
      </div>
      <div class="w-full px-lg text-xl text-left">Semantic Colors Reference</div>
      <div class="flex flex-col w-full">
        <div class="w-full flex flex-row px-lg">
          <div
            class="bg-shade-0 text-shade-100 text-center flex flex-col justify-center flex-grow aspect-square basis-0"
          >
            <div>shade-0</div>
            <div>#FFFFFF</div>
          </div>
          <div
            class="bg-shade-100 text-shade-0 text-center flex flex-col justify-center flex-grow aspect-square basis-0"
          >
            <div>shade-100</div>
            <div>#000000</div>
          </div>
          <div v-for="i in 8" :key="i" class="flex-grow aspect-square basis-0"></div>
        </div>
        <template v-for="color in ColorNamesArray" :key="color">
          <div v-if="color !== 'shade'" class="flex flex-row px-lg w-full justify-start items-center">
            <div
              v-for="i in 10"
              :key="i"
              :class="
                clsx(
                  'text-shade-0 text-center flex flex-col justify-center flex-grow aspect-square basis-0',
                  `bg-${color}-${indexToColorNumber(i)}`,
                  `text-shade-${i > 5 ? '0' : '100'}`,
                )
              "
            >
              <div>bg-{{ color }}-{{ indexToColorNumber(i) }}</div>
              <div>
                {{ colors[color][indexToColorNumber(i)] }}
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import {
  Icon,
  IconNames,
  ICONS,
  LOGO_ICONS,
  ColorNamesArray,
  SPINNABLE_ICONS,
  NewButton,
  ButtonTones,
  BUTTON_TONES,
} from "@si/vue-lib/design-system";
import { colors } from "@si/vue-lib";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import clsx from "clsx";
import { computed } from "vue";
import CheechSvg from "@/assets/images/cheech-and-chong.svg?component";
import EmptyStateIcon, { BIG_ICONS } from "@/components/EmptyStateIcon.vue";
import AppLayout from "@/components/layout/AppLayout.vue";
import DebugNavbar from "@/components/DebugNavbar.vue";
import NodeSkeleton from "@/components/NodeSkeleton.vue";

type ColorNumber = "50" | "100" | "200" | "300" | "400" | "500" | "600" | "700" | "800" | "900";

const indexToColorNumber = (i: number) => (i === 1 ? 50 : (i - 1) * 100).toString() as ColorNumber;

const buttonVariants = computed(() => [...BUTTON_TONES, "disabled"]);

const getButtonTooltip = (variant: string) => {
  if (variant === "disabled") {
    return "Disabled buttons can have explanatory tooltips";
  } else if (variant === "empty") {
    return "The empty variant is used to clear all tone related styles";
  } else if (variant === "nostyle") {
    return "The nostyle variant is used to clear ALL styles";
  } else {
    return undefined;
  }
};
</script>
