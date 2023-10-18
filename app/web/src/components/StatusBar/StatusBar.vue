<template>
  <div
    :class="
      clsx(
        'flex justify-end h-12',
        'bg-neutral-900 text-white relative border-t border-shade-100 shadow-[0_4px_4px_0_rgba(0,0,0,0.15)] z-90',
      )
    "
  >
    <StatusBarDiffSummary v-if="!changeSetStore.headSelected" />
    <StatusBarQualificationSummary />
  </div>
  <!--  -->
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import clsx from "clsx";
import { useChangeSetsStore } from "@/store/change_sets.store";

import StatusBarDiffSummary from "./StatusBarDiffSummary.vue";
import StatusBarQualificationSummary from "./StatusBarQualificationSummary.vue";

// override theme to be always dark within status bar

const changeSetStore = useChangeSetsStore();
</script>

<style scoped></style>
