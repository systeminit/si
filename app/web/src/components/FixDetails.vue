<template>
  <span>
    <button
      class="underline text-action-400 font-bold"
      @click="modalRef.open()"
    >
      View Details
    </button>

    <Modal ref="modalRef" size="2xl">
      <template #title>
        <span class="flex items-center" :title="message.join('\n')">
          <Icon
            :name="icon.name"
            class="pr-2"
            :class="icon.class"
            size="lg"
            :title="`Health: ${health}`"
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
          <p v-if="message.length === 0">Health {{ health }}</p>
        </span>
      </template>

      <!-- modal default content-->
      <div
        class="flex flex-col my-2 p-2 border border-warning-600 text-warning-500 rounded"
      >
        <b>Logs: </b>
        <p
          v-for="(log, index) in details"
          :key="index"
          class="text-sm break-all"
        >
          {{ log }}
        </p>
      </div>
    </Modal>
  </span>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { Icon, IconNames, Modal } from "@si/vue-lib/design-system";
import { ResourceHealth } from "@/api/sdf/dal/resource";

const props = defineProps<{
  health: ResourceHealth;
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
