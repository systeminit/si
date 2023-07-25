<template>
  <div
    class="rounded-xl bg-shade-0 dark:bg-shade-100 border border-neutral-300 dark:border-neutral-400 flex flex-col items-center py-sm m-sm min-w-[210px] max-w-[250px]"
  >
    <div
      class="rounded-full border border-neutral-400 h-7 flex flex-row items-center"
    >
      <div class="text-center font-bold text-neutral-500 text-lg px-xs">
        {{ number }}
      </div>
    </div>
    <div class="w-52 pb-sm">
      <EmptyStateIcon :name="iconName" />
    </div>
    <div class="flex flex-row items-center text-neutral-400 pb-xs">
      <Icon :name="number === 3 ? 'beaker' : 'diagram'" />
      <div class="whitespace-nowrap font-bold pl-xs">
        <template v-if="number === 1">Drag & Drop</template>
        <template v-if="number === 2">Connect Edges</template>
        <template v-if="number === 3">Customize Functions</template>
        <template v-if="number === 4">Apply Changes</template>
      </div>
    </div>
    <div class="text-xs italic text-center text-neutral-400 px-xs">
      <template v-if="number === 1">
        Drag assets to the canvas to start simulating your infrastructure
      </template>
      <template v-if="number === 2">
        Connect edges from socket to socket to build up your infrastructure
      </template>
      <template v-if="number === 3">
        Customize configuration and behavior using functions in the Customize
        tab
      </template>
      <template v-if="number === 4">
        Apply the changes that you have simulated to the real world!
      </template>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { computed } from "vue";
import EmptyStateIcon, { EMPTY_STATE_ICON_NAMES } from "../EmptyStateIcon.vue";

const props = defineProps({
  number: { type: Number, required: true },
});

const iconName = computed(() => {
  const names: EMPTY_STATE_ICON_NAMES[] = [
    "no-components",
    "connect-edges",
    "customize",
    "apply-changes",
  ];

  return names[props.number - 1] || "no-components";
});
</script>
