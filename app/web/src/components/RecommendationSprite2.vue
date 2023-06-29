<template>
  <Collapsible
    as="div"
    contentAs="ul"
    :defaultOpen="false"
    hideBottomBorderWhenOpen
  >
    <template #label>
      <div
        class="flex flex-row gap-2 items-center text-sm relative min-w-0 w-full justify-end"
        :class="props.class"
      >
        <div class="flex flex-col min-w-0 grow">
          <span class="font-bold truncate flex flex-row">
            <span class="grow">{{ props.recommendation.name }}</span>
            <Switch
              :id="`${props.recommendation.confirmationAttributeValueId}-${props.recommendation.actionKind}`"
              v-model="inputValue"
              :class="inputValue ? 'bg-blue-600' : 'bg-gray-200'"
              class="relative inline-flex h-5 w-8 items-center rounded-full mt-1 mr-3"
            >
              <span
                :class="inputValue ? 'translate-x-4' : 'translate-x-1'"
                class="inline-block h-3 w-3 transform rounded-full bg-white transition"
              />
            </Switch>
          </span>
          <span class="text-neutral-400 truncate">
            <!-- TODO(wendy) - sometimes the component name doesn't load properly? not sure why -->
            {{
              props.recommendation.componentName
                ? props.recommendation.componentName
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
        <div
          v-if="
            props.recommendation.lastFix &&
            props.recommendation.lastFix.status === 'failure'
          "
          class="pb-xs text-destructive-500"
        >
          <div class="font-bold">Last attempt failed!</div>
          <div
            v-if="props.recommendation.lastFix.startedAt"
            class="italic text-xs"
          >
            Started At:
            <Timestamp
              :date="new Date(props.recommendation.lastFix.startedAt)"
              size="long"
            />
          </div>
          <div
            v-if="props.recommendation.lastFix.finishedAt"
            class="italic text-xs"
          >
            Failed At:
            <Timestamp
              :date="new Date(props.recommendation.lastFix.finishedAt)"
              size="long"
            />
          </div>
        </div>
        <div class="flex flex-row justify-between text-sm">
          <div class="flex flex-col">
            <div class="font-bold">Cloud Provider:</div>
            <div>
              {{
                props.recommendation.provider
                  ? props.recommendation.provider
                  : "unknown"
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
            <div>{{ props.recommendation.actionKind }}</div>
          </div>
        </div>
      </div>
    </template>
  </Collapsible>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import clsx from "clsx";
import {
  Collapsible,
  Timestamp,
  themeClasses,
} from "@si/vue-lib/design-system";
import { Switch } from "@headlessui/vue";
import { Recommendation } from "@/store/fixes.store";

const props = defineProps({
  recommendation: { type: Object as PropType<Recommendation>, required: true },
  class: { type: String },
  selected: { type: Boolean, default: false },
});

const inputValue = computed<boolean | undefined>({
  get() {
    return props.selected;
  },
  set(value) {
    emit("toggle", !!value);
  },
});

const emit = defineEmits<{
  (e: "toggle", checked: boolean): void;
}>();
</script>
