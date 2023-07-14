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
      <template v-if="appTheme === 'light'">
        <ConnectEdgesLight v-if="number === 2" />
        <CustomizeFunctionsLight v-else-if="number === 3" />
        <ApplyChangesLight v-else-if="number === 4" />
        <NoComponentsLight v-else />
      </template>
      <template v-else>
        <ConnectEdgesDark v-if="number === 2" />
        <CustomizeFunctionsDark v-else-if="number === 3" />
        <ApplyChangesDark v-else-if="number === 4" />
        <NoComponentsDark v-else />
      </template>
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
      Drag assets to the canvas to start simulating your infrastructre
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon, useTheme } from "@si/vue-lib/design-system";
import NoComponentsLight from "@/assets/images/no-components__light.svg?component";
import NoComponentsDark from "@/assets/images/no-components__dark.svg?component";
import CustomizeFunctionsLight from "@/assets/images/customize-functions__light.svg?component";
import CustomizeFunctionsDark from "@/assets/images/customize-functions__dark.svg?component";
import ConnectEdgesLight from "@/assets/images/connect-edges__light.svg?component";
import ConnectEdgesDark from "@/assets/images/connect-edges__dark.svg?component";
import ApplyChangesLight from "@/assets/images/apply-changes__light.svg?component";
import ApplyChangesDark from "@/assets/images/apply-changes__dark.svg?component";

const { theme: appTheme } = useTheme();

defineProps({
  number: { type: Number, required: true },
});
</script>
