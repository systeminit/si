<template>
  <SiTabGroup :selected-index="0">
    <template #tabs>
      <SiTabHeader :key="0" class="text-uppercase">Apply</SiTabHeader>
    </template>
    <template #panels>
      <TabPanel :key="0" class="h-full overflow-hidden flex flex-col">
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
          <VButton2
            :disabled="
              selectedRecommendations.length < 1 ||
              fixesStore.populatingFixes ||
              (fixesStore.runningFixBatch !== undefined &&
                fixesStore.completedFixesOnRunningBatch.length <
                  fixesStore.fixesOnRunningBatch.length)
            "
            icon="tools"
            tone="action"
            @click="runFixes"
          >
            Apply
          </VButton2>
        </div>
        <div
          :class="
            clsx(
              'flex-none flex flex-row p-4 w-full items-center justify-between border-b',
              themeClasses('border-neutral-200', 'border-neutral-600'),
            )
          "
        >
          <div class="mr-2 whitespace-nowrap">Recommendations</div>
          <div
            v-if="creationRecommendations.length > 0"
            class="py-1 px-2 rounded whitespace-nowrap flex flex-row items-center text-success-500 bg-success-50 dark:text-success-100 dark:bg-success-500"
          >
            <Icon
              name="tools"
              size="xs"
              class="text-success-500 dark:text-success-100"
            />
            <span class="pl-1">{{ creationRecommendations.length }}</span>
          </div>
          <div
            v-if="genericRecommendations.length > 0"
            class="py-1 px-2 rounded whitespace-nowrap flex flex-row items-center text-destructive-500 bg-destructive-50 dark:text-destructive-100 dark:bg-destructive-500"
          >
            <Icon
              name="tools"
              size="xs"
              class="text-destructive-500 dark:text-destructive-100"
            />
            <span class="pl-1">{{ creationRecommendations.length }}</span>
          </div>
          <Icon
            v-if="fixesStore.populatingFixes"
            name="loader"
            size="md"
            class="text-action-500 dark:text-action-100"
          />
        </div>
        <div class="relative w-full h-full overflow-y-auto">
          <TransitionGroup
            tag="ul"
            enter-active-class="duration-500 ease-out"
            enter-from-class="opacity-0"
            enter-to-class="opacity-100"
            leave-active-class="duration-300 ease-in"
            leave-from-class="opacity-100 "
            leave-to-class="opacity-0"
          >
            <li
              v-for="recommendation in creationRecommendations"
              :key="`${recommendation.id}-${recommendation.recommendation}`"
            >
              <RecommendationSprite
                :recommendation="recommendation"
                :selected="
                  recommendationSelection[
                    `${recommendation.id}-${recommendation.recommendation}`
                  ]
                "
                @toggle="
                  (c) => {
                    recommendationSelection[
                      `${recommendation.id}-${recommendation.recommendation}`
                    ] = c;
                  }
                "
              />
            </li>
          </TransitionGroup>

          <Transition
            enter-active-class="delay-300 duration-200 ease-out"
            enter-from-class="opacity-0"
            enter-to-class="opacity-100"
            leave-active-class="duration-200 ease-in"
            leave-from-class="opacity-100 "
            leave-to-class="opacity-0"
          >
            <div
              v-if="creationRecommendations.length === 0"
              class="absolute top-0 p-4"
            >
              <img
                v-if="fixesStore.allRecommendations.length > 0"
                src="../assets/images/WhiskersTriumphV1.png"
                alt="Whiskers the cat, relaxing"
              />
              <img
                v-else
                src="../assets/images/WhiskersPensiveV1.png"
                alt="Whiskers the cat, looking at you"
              />
            </div>
          </Transition>
        </div>
      </TabPanel>
    </template>
  </SiTabGroup>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import { reactive, ref, computed, onBeforeUnmount, onBeforeMount } from "vue";
import { addSeconds } from "date-fns";
import clsx from "clsx";
import SiTabGroup from "@/components/SiTabGroup.vue";
import SiTabHeader from "@/components/SiTabHeader.vue";
import SiSearch from "@/components/SiSearch.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import { useFixesStore } from "@/store/fixes/fixes.store";
import { themeClasses } from "@/ui-lib/theme_tools";
import RecommendationSprite from "@/components/RecommendationSprite.vue";

const selectAll = (checked: boolean) => {
  for (const recommendation of creationRecommendations.value) {
    recommendationSelection[
      `${recommendation.id}-${recommendation.recommendation}`
    ] = checked;
  }
};

const allSelected = computed(() => {
  if (creationRecommendations.value.length === 0) return false;
  else if (
    selectedRecommendations.value.length ===
    creationRecommendations.value.length
  )
    return true;
  return false;
});

const fixesStore = useFixesStore();
const recommendations = computed(() =>
  fixesStore.allRecommendations.filter(
    (recommendation) =>
      recommendation.finishedAt === undefined ||
      recommendation.finishedAt > addSeconds(currentTime.value, -2),
  ),
);
const creationRecommendations = computed(() =>
  recommendations.value.filter((r) => r.recommendationKind === "create"),
);

const genericRecommendations = computed(() =>
  recommendations.value.filter((r) => r.recommendationKind === "other"),
);

const recommendationSelection: Record<string, boolean> = reactive({});
const selectedRecommendations = computed(() => {
  return creationRecommendations.value.filter((recommendation) => {
    return (
      recommendationSelection[
        `${recommendation.id}-${recommendation.recommendation}`
      ] && recommendation.status === "unstarted"
    );
  });
});

const runFixes = () => {
  fixesStore.EXECUTE_FIXES_FROM_RECOMMENDATIONS(selectedRecommendations.value);
};

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
