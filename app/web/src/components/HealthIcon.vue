<template>
  <span class="flex flex-row items-center">
    <Icon
      :name="icon.name"
      class="pr-2 mr-2"
      :class="icon.class"
      size="lg"
      :title="`Health: ${health}`"
    />

    <span
      class="flex flex-col w-full h-full break-words"
      :title="message.join('\n')"
    >
      <strong
        v-for="(singleMessage, index) in message"
        :key="singleMessage"
        class="mt-1 ml-1"
      >
        {{ singleMessage }}
        <button
          v-if="index === 0 && details.length > 0"
          class="underline text-action-400"
          @click="modalRef.open()"
        >
          View Details
        </button>
      </strong>
      <strong v-if="message.length === 0">Health {{ health }}</strong>
    </span>

    <Modal ref="modalRef" size="2xl">
      <template #title>
        <span class="flex" :title="message.join('\n')">
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
import Icon from "@/ui-lib/icons/Icon.vue";
import { IconNames } from "@/ui-lib/icons/icon_set";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import Modal from "@/ui-lib/modals/Modal.vue";

const props = defineProps<{
  health: ResourceHealth;
  message: string[];
  viewDetails: string[];
}>();

const details = computed(() => {
  return props.viewDetails.flatMap((d) => d.split("\\n"));
});

const icon = computed(() => {
  switch (props.health) {
    case ResourceHealth.Ok:
      return { name: "check-square" as IconNames, class: "text-success-500" };
    case ResourceHealth.Warning:
      return { name: "alert-square" as IconNames, class: "text-warning-500" };
    case ResourceHealth.Error:
      return { name: "x-square" as IconNames, class: "text-destructive-500" };
    case ResourceHealth.Unknown:
    default:
      return { name: "help-circle" as IconNames, class: "text-neutral-300" };
  }
});

const modalRef = ref();
</script>
