<template>
  <StatusBarTab
    class="cursor-pointer"
    :selected="props.selected"
    :click="openModal"
  >
    <template #icon>
      <Icon class="text-destructive-700" name="read-only" />
    </template>
    <template #name>Conflicts</template>
    <template #summary>
      <StatusBarTabPill
        v-if="statusStore.conflicts.length > 0"
        class="bg-destructive-100 text-destructive-700 font-bold"
      >
        <div @click="openModal">{{ statusStore.conflicts.length }}</div>
      </StatusBarTabPill>
      <Modal ref="modalRef" title="Conflicts" noExit>
        <div>
          <div class="flex flex-row gap-sm items-center">
            <Icon
              name="alert-circle"
              class="text-warning-600 content-center ml-md"
              size="lg"
            />
            <p class="grow py-md">
              There are {{ statusStore.conflicts.length }} conflict(s) in this
              change set ({{ changeSetsStore.selectedChangeSet?.name }}) that
              need to be resolved before you can apply.
            </p>
          </div>
          <div
            v-show="show"
            class="max-h-[50vh] overflow-hidden overflow-y-auto"
          >
            <pre
              v-for="conflict in statusStore.conflicts"
              :key="conflict"
              class="text-sm"
              >{{ conflict }}</pre
            >
            >
          </div>
          <div class="flex flex-row gap-sm items-center">
            <VButton
              label="View conflict message"
              tone="empty"
              variant="solid"
              class="grow text-action-300 dark:hover:text-white hover:text-black hover:underline"
              @click="() => (show = !show)"
            ></VButton>
            <VButton
              class="grow text-action-300 dark:hover:text-white hover:text-black hover:underline"
              label="Close"
              tone="empty"
              variant="solid"
              @click="modalRef.close()"
            ></VButton>
          </div>
        </div>
      </Modal>
    </template>
  </StatusBarTab>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Icon, Modal, VButton } from "@si/vue-lib/design-system";
import { useStatusStore } from "@/store/status.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import StatusBarTabPill from "./StatusBarTabPill.vue";
import StatusBarTab from "./StatusBarTab.vue";

const modalRef = ref();
const show = ref(false);

const openModal = () => {
  modalRef.value.open();
};

const props = defineProps({
  selected: Boolean,
});

const statusStore = useStatusStore();
const changeSetsStore = useChangeSetsStore();
</script>

<style type="less">
pre {
  white-space: pre-wrap; /* Since CSS 2.1 */
}
</style>
