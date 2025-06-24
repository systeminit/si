<template>
  <div class="w-[50svh] mx-auto mt-[15svh]">
    <h1 class="text-center text-2xl">Welcome!</h1>
    <h2 class="text-center text-xl">Take a load off</h2>
    <h3 v-if="niflheimTotal > 0" class="text-center text-lg">
      {{ niflheimComplete }} change set(s) loaded of {{ niflheimTotal }}
    </h3>
    <Icon class="mx-auto" name="loader" size="full" />
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { computed } from "vue";
import { muspelheimStatuses } from "@/store/realtime/heimdall";

const niflheimTotal = computed(
  () => Object.keys(muspelheimStatuses.value).length,
);
const niflheimComplete = computed(
  () =>
    Object.values(muspelheimStatuses.value).filter((status) => status === true)
      .length,
);
</script>
