<template>
  <div
    :class="
      clsx(
        'flex items-center relative p-sm',
        'border rounded cursor-pointer',
        variant === 'primary' && primaryClasses,
        variant === 'secondary' && 'grow h-[80px] flex-row gap-xs',
        themeClasses(
          'text-neutral-800 bg-neutral-100 border-neutral-300 hover:text-black hover:border-black',
          'text-neutral-200 bg-neutral-900 border-neutral-600 hover:text-white hover:border-white',
        ),
      )
    "
    @click.stop.prevent="emit('select', provider)"
  >
    <div v-if="variant === 'primary'" class="grow flex flex-row items-center">
      <div class="relative flex flex-col items-center">
        <Icon :name="icon" size="none" :class="iconClasses" />
      </div>
    </div>
    <Icon v-else :name="icon" size="lg" :class="iconClasses" />
    <TextPill v-if="beta" :class="betaClasses"> Beta </TextPill>
    <div>
      {{ provider }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon, TextPill, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";
import { tw } from "@si/vue-lib";
import { pickBrandIconByStringPermissive } from "./util";
import { windowWidthReactive } from "./logic_composables/emitters";

export type Provider =
  | "AWS"
  | "Azure"
  | "Hetzner"
  | "DigitalOcean"
  | "Google Cloud Platform";

export type OnboardingProviderTileVariant = "primary" | "secondary";

const props = withDefaults(
  defineProps<{
    provider: Provider;
    variant?: OnboardingProviderTileVariant;
    beta?: boolean;
  }>(),
  {
    variant: "primary",
  },
);

const icon = computed(() => {
  if (props.provider === "Hetzner" && props.variant === "primary") {
    // special logotype icon for Hetzner
    return "hetzner-logotype";
  }

  return pickBrandIconByStringPermissive(props.provider);
});

const iconClasses = computed(() => {
  // Secondary tile classes
  if (props.variant === "secondary") {
    return tw`flex-none`;
  }

  // Primary tile classes
  if (props.provider === "Hetzner") {
    // special logotype icon for Hetzner
    return tw`w-36 h-16`;
  }
  return tw`w-16 h-16`;
});

const betaClasses = computed(() =>
  clsx(
    props.variant === "primary"
      ? tw`absolute top-xs left-xs`
      : tw`order-last ml-auto`,
    tw`border text-xs`,
    themeClasses(tw`border-action-300 bg-action-100`, tw`border-action-500 bg-action-900`),
  ),
);

const primaryClasses = computed(() =>
  clsx(
    tw`h-[280px] flex-col gap-sm`,
    windowWidthReactive.value > 900 ? tw`w-[220px]` : tw`w-[200px]`,
  ),
);

const emit = defineEmits<{
  (e: "select", provider: Provider): void;
}>();
</script>
