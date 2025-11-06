<template>
  <div
    :class="
      clsx(
        'flex flex-col items-center w-[200px] h-[270px]',
        'border rounded cursor-pointer',
        themeClasses(
          'text-neutral-800 bg-neutral-100 border-neutral-300 hover:text-black hover:border-black',
          'text-neutral-200 bg-neutral-900 border-neutral-600 hover:text-white hover:border-white',
        ),
      )
    "
    @click.stop.prevent="emit('select', provider)"
  >
    <div class="grow flex flex-row items-center">
      <Icon
        :name="pickBrandIconByString(provider)"
        size="none"
        class="w-16 h-16"
      />
    </div>
    <div class="py-sm">
      {{ provider }}
      <template v-if="beta">(beta)</template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { pickBrandIconByString } from "./util";

export type Provider = "AWS" | "Azure";

defineProps<{
  provider: Provider;
  beta?: boolean;
}>();

const emit = defineEmits<{
  (e: "select", provider: Provider): void;
}>();
</script>
