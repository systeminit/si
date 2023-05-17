<template>
  <ScrollArea>
    <div class="flex flex-col h-full">
      Diagram Outline
      <div class="grow relative">
        <ComponentOutline />
      </div>
    </div>
  </ScrollArea>
</template>

<script lang="ts" setup>
import { reactive, ref, computed, onBeforeUnmount, onBeforeMount } from "vue";
import clsx from "clsx";
import {
  TabGroup,
  TabGroupItem,
  VButton,
  Icon,
  VormInput,
  themeClasses,
  ScrollArea,
} from "@si/vue-lib/design-system";
import SiSearch from "@/components/SiSearch.vue";
import { useFixesStore } from "@/store/fixes.store";
import { useStatusStore } from "@/store/status.store";
import RecommendationSprite from "@/components/RecommendationSprite.vue";
import ComponentOutline from "@/components/ComponentOutline/ComponentOutline.vue";

const selectAll = (checked: boolean) => {
  for (const recommendation of recommendations.value) {
    recommendationSelection[
      `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`
    ] = checked;
  }
};

const allSelected = computed(() => {
  if (recommendations.value.length === 0) return false;
  else if (
    selectedRecommendations.value.length === recommendations.value.length
  )
    return true;
  return false;
});

const recommendations = computed(() => fixesStore.recommendations);

const statusStore = useStatusStore();
const fixesStore = useFixesStore();
const creationRecommendations = computed(() =>
  recommendations.value.filter((r) => r.actionKind === "create"),
);

const genericRecommendations = computed(() =>
  recommendations.value.filter((r) => r.actionKind === "other"),
);

const destructionRecommendations = computed(() =>
  recommendations.value.filter((r) => r.actionKind === "delete"),
);

const recommendationSelection: Record<string, boolean> = reactive({});
const selectedRecommendations = computed(() => {
  return recommendations.value.filter((recommendation) => {
    return (
      recommendationSelection[
        `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`
      ] && !recommendation.hasRunningFix
    );
  });
});

const runFixes = () => {
  fixesStore.EXECUTE_FIXES_FROM_RECOMMENDATIONS(selectedRecommendations.value);
};

const confirmationsInFlight = computed(() => {
  for (const c of fixesStore.confirmations) {
    if (c.status === "neverStarted") {
      return true;
    }
  }
  return false;
});

const disableApply = computed(
  () =>
    selectedRecommendations.value.length < 1 ||
    statusStore.globalStatus.isUpdating ||
    fixesStore.populatingFixes ||
    fixesStore.runningFixBatch !== undefined,
);

const currentTime = ref(new Date());
let dateIntervalId: Timeout;

onBeforeMount(() => {
  dateIntervalId = setInterval(() => {
    currentTime.value = new Date();
  }, 500);
});

onBeforeUnmount(() => {
  clearInterval(dateIntervalId);
});
</script>
