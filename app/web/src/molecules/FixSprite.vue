<template>
  <SiCollapsible :default-open="false" hide-bottom-border>
    <template #label>
      <div
        class="flex flex-row items-center gap-2.5 text-sm relative"
        :class="classes"
      >
        <VormInput
          v-if="fix.status === 'unstarted'"
          :model-value="selected"
          type="checkbox"
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
          :class="clsx('shrink-0', statusIconProps.color)"
          size="lg"
        />

        <Icon
          v-if="fix.status !== 'success'"
          name="tools"
          size="lg"
          class="text-destructive-500 shrink-0"
        />
        <div class="w-full text-ellipsis whitespace-nowrap overflow-hidden">
          {{ fix.name }}
        </div>
      </div>
    </template>
    <template #default>
      <div class="pl-8 pr-2 py-4">
        <span class="font-bold">Recommendation: </span>{{ fix.recommendation }}
      </div>
    </template>
  </SiCollapsible>
</template>

<script setup lang="ts">
import { Ref, computed, PropType } from "vue";
import clsx from "clsx";
import Icon, { IconNames } from "@/ui-lib/Icon.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import SiCollapsible from "@/organisms/SiCollapsible.vue";
import { Fix } from "@/store/fixes/fixes.store";

const props = defineProps({
  fix: { type: Object as PropType<Fix>, required: true },
  class: { type: String },
  selected: { type: Boolean, default: false },
});

const classes = computed(() => props.class);

const emit = defineEmits<{
  (e: "toggle", checked: boolean): void;
}>();

const statusIconProps: Ref<{ name: IconNames; color: string }> = computed(
  () => {
    switch (props.fix.status) {
      case "failure":
        return { name: "x-circle", color: "text-destructive-500" };
      case "success":
        return { name: "check-circle", color: "text-success-500" };
      default:
        return { name: "loader", color: "text-action-300" };
    }
  },
);
</script>
