<template>
  <div
    class="w-full h-full flex flex-col items-center relative overflow-hidden dark:bg-neutral-800 dark:text-shade-0 bg-neutral-50 text-neutral-900"
  >
    <span class="flex flex-row mt-10 font-bold text-3xl">Dev Dashboard</span>
    <div class="flex flex-row mt-10">
      <span class="font-bold">Current Git SHA (Local):&nbsp;</span>
      <a v-if="gitShaLink" :href="gitShaLink" class="text-action-400">
        <span>{{ gitSha }}</span>
      </a>
      <span v-else-if="gitSha">{{ gitSha }}</span>
      <span v-else>null</span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { DevService } from "@/service/dev";

const currentGitSha = DevService.useCurrentGitSha();
const gitSha = computed(() => currentGitSha?.value?.sha || null);
const gitShaLink = computed(() =>
  gitSha.value
    ? `https://github.com/systeminit/si/commit/${gitSha.value}`
    : null,
);
</script>
