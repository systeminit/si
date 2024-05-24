<template>
  <div
    :class="
      clsx(
        'flex justify-end',
        'bg-neutral-900 text-white relative border-t border-shade-100 shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] z-90',
      )
    "
  >
    <div class="flex text-sm items-center pl-xs mr-lg w-full">
      System&nbsp;Initiative
    </div>
    <div
      v-if="statusStore.conflicts.length > 0"
      class="border-l border-shade-100"
    >
      <StatusBarConflictSummary />
    </div>
    <div class="border-l border-shade-100">
      <StatusBarDiffSummary v-if="!changeSetStore.headSelected" />
    </div>
    <div v-if="changeSetStore.headSelected" class="border-i border-shade-100">
      <StatusBarResourceSummary />
    </div>
    <div class="border-l border-shade-100">
      <StatusBarQualificationSummary />
    </div>
  </div>
  <!--  -->
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { useChangeSetsStore } from "@/store/change_sets.store";

import { useStatusStore } from "@/store/status.store";
import StatusBarConflictSummary from "./StatusBarConflictSummary.vue";
import StatusBarDiffSummary from "./StatusBarDiffSummary.vue";
import StatusBarResourceSummary from "./StatusBarResourceSummary.vue";
import StatusBarQualificationSummary from "./StatusBarQualificationSummary.vue";

// override theme to be always dark within status bar

const changeSetStore = useChangeSetsStore();
const statusStore = useStatusStore();
</script>

<style scoped></style>
