<template>
  <SiCollapsible
    as="li"
    class="w-full"
    content-as="ul"
    :default-open="false"
    hide-bottom-border-when-open
  >
    <template #prefix>
      <VormInput
        v-if="fix.status === 'unstarted'"
        :model-value="selected"
        type="checkbox"
        class="pl-1"
        no-label
        @click.stop
        @update:model-value="
          (c) => {
            emit('toggle', c);
          }
        "
      />
      <Icon
        v-else-if="fix.status === 'running'"
        name="loader"
        :class="clsx('shrink-0', statusIconProps.color)"
        size="lg"
      />
      <Icon
        v-else
        :name="statusIconProps.name"
        :class="clsx('shrink-0', statusIconProps.color)"
        size="lg"
      />
    </template>
    <template #label>
      <div
        class="flex flex-row items-center gap-2.5 text-sm relative"
        :class="classes"
      >
        <Icon
          v-if="fix.status !== 'success'"
          name="tools"
          size="md"
          class="text-destructive-500 shrink-0"
        />
        <div class="w-full text-ellipsis overflow-hidden line-clamp-2">
          {{ fix.name }}
        </div>
      </div>
    </template>
    <template #default>
      <div class="flex flex-row justify-around text-sm">
        <div class="flex flex-col">
          <div class="font-bold">Cloud Provider:</div>
          <div>idk</div>
        </div>
        <div class="flex flex-col">
          <div class="font-bold">Environment:</div>
          <div>idk</div>
        </div>
      </div>
      <div
        :class="
          clsx(
            'pl-8 pr-2 py-4 text-sm border-b',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <div class="flex flex-col">
          <div class="font-bold">Recommendation:</div>
          <div>{{ fix.recommendation }}</div>
        </div>
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
import { themeClasses } from "@/ui-lib/theme_tools";

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
