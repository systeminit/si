<template>
  <div class="flex-none w-6 h-6">
    <FixCardIconButton
      tooltip="see logs"
      icon="logs-pop"
      iconHover="logs-pop-square"
      :selected="modalRef?.isOpen"
      @click="modalRef.open()"
    />

    <Modal ref="modalRef" size="2xl">
      <template #title>
        <span class="flex items-center" :title="message.join('\n')">
          <Icon
            :name="icon.name"
            class="pr-2"
            :class="icon.class"
            size="lg"
            :title="`Health: ${health ?? 'unknown'}`"
          />

          <span class="flex flex-col">
            <p
              v-for="singleMessage in message"
              :key="singleMessage"
              class="mt-1 ml-1"
            >
              {{ singleMessage }}
            </p>
          </span>
          <p v-if="message.length === 0">Health {{ health ?? "unknown" }}</p>
        </span>
      </template>

      <div
        class="text-lg font-bold border-warning-600 text-warning-600 dark:text-warning-500 pb-xs"
      >
        {{ details.length === 0 ? "No Logs" : "Logs:" }}
      </div>

      <!-- modal default content-->
      <div
        class="border border-warning-600 text-warning-600 dark:text-warning-500 rounded overflow-hidden"
      >
        <div class="flex flex-col gap-sm max-h-[80vh] overflow-y-auto p-xs">
          <p
            v-for="(log, index) in details"
            :key="index"
            class="text-sm break-all"
          >
            {{ log }}
          </p>
        </div>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { Icon, IconNames, Modal } from "@si/vue-lib/design-system";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import FixCardIconButton from "./FixCardIconButton.vue";

const props = defineProps<{
  health: ResourceHealth | null;
  message: string[];
  details: string[];
}>();

const details = computed(() => {
  return props.details.flatMap((d) => d.split("\\n"));
});

const icon = computed(() => {
  switch (props.health) {
    case ResourceHealth.Ok:
      return { name: "check2" as IconNames, class: "text-success-500" };
    case ResourceHealth.Warning:
      return { name: "alert-triangle" as IconNames, class: "text-warning-500" };
    case ResourceHealth.Error:
      return {
        name: "alert-triangle" as IconNames,
        class: "text-destructive-500",
      };
    case ResourceHealth.Unknown:
    default:
      return {
        name: "question-circle" as IconNames,
        class: "text-neutral-300",
      };
  }
});

const modalRef = ref();
</script>
