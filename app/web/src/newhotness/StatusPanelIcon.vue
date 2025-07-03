<template>
  <Transition
    enterActiveClass="transition-opacity duration-300"
    enterFromClass="opacity-0"
    leaveActiveClass="transition-opacity duration-500"
    leaveToClass="opacity-0"
    appear
  >
    <!-- We may want to make this "invisible" to keep the space in the future. -->
    <div
      v-if="show"
      v-tooltip="
        status === 'syncing' ? 'Syncing changes' : 'Everything is synced'
      "
      class="bg-neutral-600 rounded p-2xs"
    >
      <Icon
        :name="
          status === 'syncing'
            ? 'refresh-carbon-active'
            : 'check-circle-outline'
        "
        size="sm"
      />
    </div>
  </Transition>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { ref, watch } from "vue";

const props = defineProps<{ status: "syncing" | "synced" | undefined }>();

const showSyncedMs = 5000;
const show = ref<boolean>(false);

watch(
  () => props.status,
  (newStatus) => {
    if (!newStatus) {
      show.value = false;
    } else if (newStatus === "syncing") {
      show.value = true;
    } else if (newStatus === "synced") {
      show.value = true;
      setTimeout(() => {
        show.value = false;
        emit("faded");
      }, showSyncedMs);
    }
  },
  { immediate: true },
);

const emit = defineEmits<{
  (e: "faded"): void;
}>();
</script>
