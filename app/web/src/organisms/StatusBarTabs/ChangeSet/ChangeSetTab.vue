<template>
  <StatusBarTab :selected="props.selected">
    <template #icon><ClockIcon class="text-white" /></template>
    <template #name>{{ title }}</template>
    <template #summary>
      <StatusBarTabPill v-if="total > 0">
        <span class="font-bold">Total:&nbsp; {{ total }}</span>
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="added > 0"
        class="bg-success-100 text-success-700 font-bold"
      >
        + {{ added }}
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="modified > 0"
        class="bg-warning-100 text-warning-700 font-bold"
      >
        ~ {{ modified }}
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="deleted > 0"
        class="bg-destructive-100 text-destructive-700 font-bold"
      >
        - {{ deleted }}
      </StatusBarTabPill>
    </template>
  </StatusBarTab>
</template>

<script setup lang="ts">
import { ClockIcon } from "@heroicons/vue/solid";
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { computed, ref } from "vue";
import { untilUnmounted } from "vuse-rx";

const props = defineProps<{
  selected: boolean;
}>();

const added = ref<number>(0);
const modified = ref<number>(0);
const deleted = ref<number>(0);
const total = computed(
  (): number => added.value + modified.value + deleted.value,
);
const title = computed((): string => {
  if (total.value > 0) {
    return "Changes";
  }
  return "No Changes yet...";
});

untilUnmounted(ChangeSetService.getStats()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
  } else {
    // Reset each counter in case the stat(s) are empty.
    let tempAdded = 0;
    let tempModified = 0;
    let tempDeleted = 0;

    for (const statsGroup of response.componentStats.stats) {
      if (statsGroup.componentStatus === "added") {
        tempAdded += 1;
      } else if (statsGroup.componentStatus === "modified") {
        tempModified += 1;
      } else {
        tempDeleted += 1;
      }
    }

    added.value = tempAdded;
    modified.value = tempModified;
    deleted.value = tempDeleted;
  }
});
</script>
