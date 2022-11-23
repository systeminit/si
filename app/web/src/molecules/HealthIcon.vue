<template>
  <span class="flex flex-row whitespace-nowrap">
    <Icon
      :name="icon.name"
      class="pr-2"
      :class="icon.class"
      size="lg"
      :title="`Health: ${props.health}`"
    />

    <span class="flex flex-col" :title="props.message.join('\n')">
      <strong v-for="(message, index) in props.message" :key="message" class="mt-1 ml-1">
        {{message}}
        <button
          v-if="index === 0 && details.length > 0"
          class="underline text-action-400 ml-2"
          @click="openModal"
        >
          View Details
        </button>
      </strong>
      <strong v-if="props.message.length === 0">Health {{ props.health }}</strong>
    </span>

    <Modal size="2xl" :open="modalOpen" @close="closeModal">
      <template #title>
        <span class="flex" :title="props.message.join('\n')">
          <Icon
            :name="icon.name"
            class="pr-2"
            :class="icon.class"
            size="lg"
            :title="`Health: ${props.health}`"
          />

          <span class="flex flex-col">
            <p v-for="message in props.message" :key="message" class="mt-1 ml-1">
              {{ message }}
            </p>
	  </span>
          <p v-if="props.message.length === 0">
            Health {{ props.health }}
	  </p>
	</span>
      </template>
      <template #content>
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
      </template>
    </Modal>
  </span>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import { IconNames } from "@/ui-lib/icons/icon_set";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import Modal from "@/ui-lib/Modal.vue";

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
      break;
    case ResourceHealth.Warning:
      return { name: "alert-square" as IconNames, class: "text-warning-500" };
      break;
    case ResourceHealth.Error:
      return { name: "x-square" as IconNames, class: "text-destructive-500" };
      break;
    case ResourceHealth.Unknown:
    default:
      return { name: "help-circle" as IconNames, class: "text-neutral-300" };
      break;
  }
});

const modalOpen = ref(false);

const openModal = () => {
  modalOpen.value = true;
};

const closeModal = () => {
  modalOpen.value = false;
};
</script>
