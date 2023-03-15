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
        v-if="recommendation.isRunnable === 'yes'"
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
        v-else-if="
          recommendation.status === 'running' ||
          recommendation.isRunnable === 'running'
        "
        name="loader"
        class="flex-none mx-1 text-action-300"
        size="lg"
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
        class="flex gap-2 items-center text-sm relative min-w-0"
        :class="classes"
      >
        <Icon
          :name="recommendationIcon(recommendation.actionKind)"
          size="md"
          :class="recommendationColor(recommendation.actionKind)"
        />
        <div class="flex flex-col min-w-0">
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
        <div class="py-4 text-sm">
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
import { Ref, computed, PropType } from "vue";
import clsx from "clsx";
import Icon from "@/ui-lib/icons/Icon.vue";
import { IconNames } from "@/ui-lib/icons/icon_set";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import { Recommendation, ActionKind } from "@/store/fixes.store";
import { themeClasses } from "@/ui-lib/theme_tools";
import SiCollapsible from "./SiCollapsible.vue";

const props = defineProps({
  recommendation: { type: Object as PropType<Recommendation>, required: true },
  class: { type: String },
  selected: { type: Boolean, default: false },
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

const recommendationIcon = (recommendationAction: ActionKind) => {
  if (recommendationAction === "create") {
    return "plus-circle";
  } else if (recommendationAction === "destroy") {
    return "minus-circle";
  } else {
    return "tilde-circle";
  }
};

const recommendationColor = (recommendationAction: ActionKind) => {
  if (recommendationAction === "create") {
    return "text-success-500 flex-none";
  } else if (recommendationAction === "destroy") {
    return "text-destructive-500 flex-none";
  } else {
    return "text-warning-500 flex-none";
  }
};
</script>
