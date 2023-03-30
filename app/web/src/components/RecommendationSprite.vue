<template>
  <SiCollapsible
    as="div"
    :class="
      clsx(
        'w-full hover:border-action-500 dark:hover:border-action-300 border border-transparent',
        selected
          ? 'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300'
          : '',
      )
    "
    content-as="ul"
    :default-open="false"
    hide-bottom-border-when-open
  >
    <template #prefix>
      <VormInput
        v-if="
          recommendation.status === 'unstarted' ||
          (recommendation.status === 'failure' && !iconDelayActive)
        "
        :model-value="selected"
        type="checkbox"
        class="flex-none pl-1"
        no-label
        @click.stop
        @update:model-value="
          (c) => {
            emit('toggle', c);
          }
        "
      />
      <Icon
        v-else
        :name="statusIconProps.name"
        :class="clsx('flex-none mx-1', statusIconProps.color)"
        size="lg"
      />
    </template>
    <template #label>
      <div
        class="flex flex-row gap-2 items-center text-sm relative min-w-0 w-full justify-end"
        :class="classes"
      >
        <!-- TODO(Wendy) - after velvet rope we can try to come up with icons for this again
          <Icon
          :name="recommendationIcon(recommendation.actionKind)"
          size="md"
          :class="recommendationColor(recommendation.actionKind)"
        /> -->
        <div class="flex flex-col min-w-0 grow">
          <span class="font-bold truncate"> {{ recommendation.name }}</span>
          <span class="text-xs text-neutral-700 dark:text-neutral-300 truncate">
            <!-- TODO(wendy) - sometimes the component name doesn't load properly? not sure why -->
            {{
              recommendation.componentName
                ? recommendation.componentName
                : "unknown"
            }}
          </span>
        </div>
        <Icon
          v-if="
            recommendation.lastFix &&
            recommendation.lastFix.status === 'failure'
          "
          name="alert-triangle"
          class="text-destructive-500"
          size="lg"
        />
      </div>
    </template>
    <template #default>
      <div
        :class="
          clsx(
            'w-full pl-[4.25rem] pr-4 border-b',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <div
          v-if="
            recommendation.lastFix &&
            recommendation.lastFix.status === 'failure'
          "
          class="pb-xs text-destructive-500"
        >
          <div class="font-bold">Last attempt failed!</div>
          <div v-if="recommendation.lastFix.startedAt" class="italic text-xs">
            Started At:
            <Timestamp
              :date="new Date(recommendation.lastFix.startedAt)"
              size="long"
            />
          </div>
          <div v-if="recommendation.lastFix.finishedAt" class="italic text-xs">
            Failed At:
            <Timestamp
              :date="new Date(recommendation.lastFix.finishedAt)"
              size="long"
            />
          </div>
        </div>
        <div class="flex flex-row justify-between text-sm">
          <div class="flex flex-col">
            <div class="font-bold">Cloud Provider:</div>
            <div>
              {{
                recommendation.provider ? recommendation.provider : "unknown"
              }}
            </div>
          </div>
          <div class="flex flex-col">
            <div class="font-bold">Environment:</div>
            <div>dev</div>
          </div>
        </div>
        <div class="py-xs text-sm">
          <div class="flex flex-col">
            <div class="font-bold">Recommendation:</div>
            <div>{{ recommendation.recommendedAction }}</div>
          </div>
        </div>
      </div>
    </template>
  </SiCollapsible>
</template>

<script setup lang="ts">
import { Ref, computed, PropType, ref, watch, onBeforeUnmount } from "vue";
import clsx from "clsx";
import {
  Timestamp,
  themeClasses,
  VormInput,
  Icon,
  IconNames,
} from "@si/vue-lib/design-system";
import { Recommendation } from "@/store/fixes.store";
import SiCollapsible from "./SiCollapsible.vue";

const props = defineProps({
  recommendation: { type: Object as PropType<Recommendation>, required: true },
  class: { type: String },
  selected: { type: Boolean, default: false },
  iconDelayAfterExec: { type: Number },
});

let delayTimeout: Timeout;
const iconDelayActive = ref(false);

watch(
  () => props.recommendation.status,
  (newVal, oldVal) => {
    if (oldVal === "running") {
      emit("toggle", false);
      iconDelayActive.value = true;
      delayTimeout = setTimeout(() => {
        iconDelayActive.value = false;
      }, props.iconDelayAfterExec);
    }
  },
);

onBeforeUnmount(() => {
  clearTimeout(delayTimeout);
});

const classes = computed(() => props.class);

const emit = defineEmits<{
  (e: "toggle", checked: boolean): void;
}>();

const statusIconProps: Ref<{ name: IconNames; color: string }> = computed(
  () => {
    switch (props.recommendation.status) {
      case "failure":
        return { name: "alert-triangle", color: "text-destructive-500" };
      case "success":
        return { name: "check-circle", color: "text-success-500" };
      default:
        return { name: "loader", color: "text-action-300" };
    }
  },
);

// const recommendationIcon = (recommendationAction: ActionKind) => {
//   if (recommendationAction === "create") {
//     return "plus-circle";
//   } else if (recommendationAction === "destroy") {
//     return "minus-circle";
//   } else {
//     return "tilde-circle";
//   }
// };

// const recommendationColor = (recommendationAction: ActionKind) => {
//   if (recommendationAction === "create") {
//     return "text-success-500 flex-none";
//   } else if (recommendationAction === "destroy") {
//     return "text-destructive-500 flex-none";
//   } else {
//     return "text-warning-500 flex-none";
//   }
// };
</script>
