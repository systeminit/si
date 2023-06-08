<template>
  <TabGroup start-selected-tab-slug="apply">
    <TabGroupItem label="Apply" slug="apply">
      <ScrollArea>
        <template #top>
          <SiSearch auto-search placeholder="search recommendations" />
          <div
            class="w-full flex-none text-neutral-400 dark:text-neutral-300 text-sm p-2 border-b dark:border-neutral-600"
          >
            Select recommendations from the list below to apply them.
          </div>
          <div
            class="w-full flex-none text-neutral-400 dark:text-neutral-300 text-sm p-2 border-b dark:border-neutral-600 flex flex-row items-center justify-between whitespace-nowrap gap-4 overflow-hidden"
          >
            <VormInput
              class="ml-2"
              type="checkbox"
              label="Select All"
              no-label
              :model-value="allSelected"
              @update:model-value="selectAll"
              >Select All
            </VormInput>
            <VButton
              :disabled="disableApply"
              icon="tools"
              tone="action"
              @click="runFixes"
            >
              Apply
            </VButton>
          </div>
          <div
            :class="
              clsx(
                'flex-none flex flex-row p-4 w-full items-center justify-between border-b overflow-hidden',
                themeClasses('border-neutral-200', 'border-neutral-600'),
              )
            "
          >
            <div class="whitespace-nowrap font-bold">Recommendations</div>
            <div class="flex flex-row grow justify-end gap-1">
              <div
                v-if="creationRecommendations.length > 0"
                class="py-1 px-2 rounded whitespace-nowrap flex flex-row items-center text-success-500 bg-success-50 dark:text-success-100 dark:bg-success-500"
              >
                <Icon
                  name="create"
                  size="xs"
                  class="text-success-500 dark:text-success-100"
                />
                <span class="pl-1">{{ creationRecommendations.length }}</span>
              </div>
              <div
                v-if="genericRecommendations.length > 0"
                class="py-1 px-2 rounded whitespace-nowrap flex flex-row items-center text-warning-500 bg-warning-50 dark:text-warning-100 dark:bg-warning-500"
              >
                <Icon
                  name="tools"
                  size="xs"
                  class="text-warning-500 dark:text-destructive-100"
                />
                <span class="pl-1">{{ genericRecommendations.length }}</span>
              </div>
              <div
                v-if="destructionRecommendations.length > 0"
                class="py-1 px-2 rounded whitespace-nowrap flex flex-row items-center text-destructive-500 bg-destructive-50 dark:text-destructive-100 dark:bg-destructive-500"
              >
                <Icon
                  name="trash"
                  size="xs"
                  class="text-destructive-500 dark:text-destructive-100"
                />
                <span class="pl-1">{{
                  destructionRecommendations.length
                }}</span>
              </div>
              <Icon
                v-if="
                  confirmationsInFlight ||
                  fixesStore.populatingFixes ||
                  fixesStore.runningFixBatch
                "
                name="loader"
                size="md"
                class="text-action-500 dark:text-action-100"
              />
            </div>
          </div>
        </template>
        <div class="relative w-full h-full overflow-y-auto">
          <TransitionGroup
            tag="ul"
            enter-active-class="duration-500 ease-out"
            enter-from-class="opacity-0"
            enter-to-class="opacity-100"
            leave-active-class="duration-300 ease-in delay-2000"
            leave-from-class="opacity-100 "
            leave-to-class="opacity-0"
          >
            <li
              v-for="recommendation in recommendations"
              :key="`${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`"
            >
              <RecommendationSprite
                :key="`${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`"
                :recommendation="recommendation"
                :selected="
                  recommendationSelection[
                    `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`
                  ]
                "
                :icon-delay-after-exec="2500"
                @toggle="
                  (c) => {
                    recommendationSelection[
                      `${recommendation.confirmationAttributeValueId}-${recommendation.actionKind}`
                    ] = c;
                  }
                "
              />
            </li>
            <li
              v-if="recommendations.length === 0"
              class="p-4 italic !delay-0 !duration-0 hidden first:block"
            >
              <div class="pb-sm">
                No recommendations are available at this time.
              </div>
              <div>
                You can go to the
                <span class="font-bold text-action-500 hover:underline">
                  <RouterLink :to="{ name: 'workspace-view' }">
                    Analyze</RouterLink
                  >
                </span>
                page to view the status of your resources.
              </div>
            </li>
          </TransitionGroup>
        </div>
      </ScrollArea>
    </TabGroupItem>
  </TabGroup>
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
import { RouterLink } from "vue-router";
import SiSearch from "@/components/SiSearch.vue";
import { useFixesStore } from "@/store/fixes.store";
import { useStatusStore } from "@/store/status.store";
import RecommendationSprite from "@/components/RecommendationSprite.vue";

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
