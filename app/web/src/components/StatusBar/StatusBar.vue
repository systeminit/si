<template>
  <div
    :class="
      clsx(
        'flex justify-end',
        'bg-neutral-900 text-white relative border-t border-shade-100 shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] z-90 h-10',
        'select-none',
      )
    "
  >
    <div class="flex text-sm items-center pl-xs mr-lg w-full">
      System&nbsp;Initiative
      <RealtimeStatusPageState />
    </div>
  </div>
  <!--  -->
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";

import RealtimeStatusPageState from "@/components/RealtimeStatusPageState.vue";
</script>

<style scoped></style>
