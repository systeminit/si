<template>
  <SiCollapsible
    as="div"
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
        v-else-if="fix.status === 'running'"
        name="loader"
        :class="clsx('flex-none pl-1', statusIconProps.color)"
        size="lg"
      />
      <Icon
        v-else
        :name="statusIconProps.name"
        :class="clsx('flex-none pl-1', statusIconProps.color)"
        size="lg"
      />
    </template>
    <template #label>
      <div
        class="flex gap-2 items-center text-sm relative min-w-0"
        :class="classes"
      >
        <Icon
          v-if="fix.status !== 'success'"
          name="tools"
          size="md"
          class="text-destructive-500 flex-none"
        />
        <div class="flex flex-col min-w-0">
          <span class="font-bold truncate"> {{ fix.name }}</span>
          <span class="text-xs text-neutral-700 dark:text-neutral-300 truncate">
            <!-- TODO(wendy) - sometimes the component name doesn't load properly? not sure why -->
            {{ fix.componentName ? fix.componentName : "unknown" }}
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
            <div>{{ fix.provider ? fix.provider : "unknown" }}</div>
          </div>
          <div class="flex flex-col">
            <div class="font-bold">Environment:</div>
            <div>dev</div>
          </div>
        </div>
        <div class="py-4 text-sm">
          <div class="flex flex-col">
            <div class="font-bold">Recommendation:</div>
            <div>{{ fix.recommendation }}</div>
          </div>
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
