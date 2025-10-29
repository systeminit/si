<template>
  <Modal
    ref="modalRef"
    :title="`Adjust Views For &quot;${componentName}&quot;`"
    :capitalizeTitle="false"
  >
    <div class="max-h-[70vh] scrollable flex flex-col gap-xs">
      <div class="text-lg">Currently in the following views -</div>
      <div
        v-for="view in availableViewListOptions.removeFromView"
        :key="view.value"
        class="flex flex-row items-center gap-xs"
      >
        <NewButton
          v-tooltip="
            onlyInOneView ? `Can't remove from last view` : `Remove from view`
          "
          icon="trash"
          tone="destructive"
          :disabled="onlyInOneView"
          @click="() => removeFromView(view.value)"
        />
        <div>{{ view.label }}</div>
      </div>
      <div class="text-lg">Can be added to the following views -</div>
      <div
        v-for="view in availableViewListOptions.addToView"
        :key="view.value"
        class="flex flex-row items-center gap-xs"
      >
        <NewButton
          v-tooltip="`Add to view`"
          icon="plus"
          tone="action"
          @click="() => addToView(view.value)"
        />
        <div>{{ view.label }}</div>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { Modal, NewButton, useModal } from "@si/vue-lib/design-system";
import { AvailableViewListOptions } from "./logic_composables/view";
import { useApi, routes } from "./api_composables";

const modalRef = ref<InstanceType<typeof Modal>>();
const { open, close } = useModal(modalRef);

const props = defineProps<{
  componentName: string;
  componentId: string;
  availableViewListOptions: AvailableViewListOptions;
}>();

const onlyInOneView = computed(
  () => props.availableViewListOptions.removeFromView.length < 2,
);

const addToViewApi = useApi();
const removeFromViewApi = useApi();
const addToView = async (viewId: string) => {
  const call = addToViewApi.endpoint(routes.ViewAddComponents, {
    viewId,
  });
  await call.post({
    componentIds: [props.componentId],
  });
};
const removeFromView = async (viewId: string) => {
  const call = removeFromViewApi.endpoint(routes.ViewEraseComponents, {
    viewId,
  });
  await call.delete({
    componentIds: [props.componentId],
  });
};

defineExpose({ open, close });
</script>
