<template>
  <!-- NOTE: the Modal CSS for height in "max" doesn't work as we might expect -->
  <Modal
    ref="modalRef"
    size="lg"
    title="Edit View"
    type="save"
    saveLabel="Done"
    @save="() => nameForm.handleSubmit()"
  >
    <label class="flex flex-row items-center relative">
      <span>View Name</span>
      <nameForm.Field name="name">
        <template #default="{ field }">
          <input
            :class="
              clsx(
                'block w-72 ml-auto border',
                themeClasses(
                  'text-black bg-white border-neutral-600 disabled:bg-neutral-100',
                  'text-white bg-black border-neutral-400 disabled:bg-neutral-900',
                ),
              )
            "
            :value="field.state.value"
            type="text"
            :disabled="wForm.bifrosting.value"
            @input="
              (e) => field.handleChange((e.target as HTMLInputElement).value)
            "
          />
        </template>
      </nameForm.Field>
      <Icon
        v-if="wForm.bifrosting.value"
        class="absolute right-2xs"
        name="loader"
        size="sm"
        tone="action"
      />
    </label>
    <template #leftButton>
      <div v-tooltip="tooltipText">
        <VButton
          label="Delete View"
          :disabled="!canDeleteView"
          :tone="canDeleteView ? 'destructive' : 'neutral'"
          @click="deleteView"
        />
      </div>
    </template>
  </Modal>
</template>

<script setup lang="ts">
import { VButton, Modal, Icon, themeClasses } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import clsx from "clsx";
import { useRoute } from "vue-router";
import { ViewId } from "@/api/sdf/dal/views";
import { useApi, routes } from "./api_composables";
import { useWatchedForm } from "./logic_composables/watched_form";

const viewId = ref<ViewId>("");
const canDeleteView = ref<boolean>(false);
const isDefaultView = ref<boolean>(false);

const tooltipText = computed(() => {
  if (isDefaultView.value) {
    return "Cannot delete the default View.";
  }
  if (canDeleteView.value) {
    return undefined;
  }
  return "Views containing one or more components cannot be deleted. To delete a view, first remove all components from it.";
});

const modalRef = ref<InstanceType<typeof Modal>>();

const route = useRoute();

const deleteViewApi = useApi();
const updateViewApi = useApi();

const deleteView = async () => {
  const call = deleteViewApi.endpoint<{ id: string }>(routes.DeleteView, {
    viewId: viewId.value,
  });
  const { req, newChangeSetId } = await call.delete({});
  if (deleteViewApi.ok(req)) {
    close();
    emit("deleted");
    if (newChangeSetId) {
      // TODO(nick): when we make editing a view not require switching to that view, make sure that
      // we re-route to what the user had selected, whether that be "all views" or their
      // previously selected view.
      deleteViewApi.navigateToNewChangeSet(
        {
          name: "new-hotness",
          params: {
            workspacePk: route.params.workspacePk,
            changeSetId: newChangeSetId,
          },
        },
        newChangeSetId,
      );
    }
  }
};

const wForm = useWatchedForm<{ name: string }>("view.update");
const formData = computed<{ name: string }>(() => {
  return { name: "" };
});

const nameForm = wForm.newForm({
  data: formData,
  onSubmit: async ({ value }) => {
    const call = updateViewApi.endpoint<{ id: string }>(routes.UpdateView, {
      viewId: viewId.value,
    });
    const { req, newChangeSetId } = await call.put<{ name: string }>({
      name: value.name,
    });
    if (updateViewApi.ok(req)) {
      close();
      if (newChangeSetId) {
        // TODO(nick): when we make editing a view not require switching to that view, make sure that
        // we re-route to what the user had selected, whether that be "all views" or their
        // previously selected view.
        updateViewApi.navigateToNewChangeSet(
          {
            name: "new-hotness",
            params: {
              workspacePk: route.params.workspacePk,
              changeSetId: newChangeSetId,
            },
          },
          newChangeSetId,
        );
      }
    }
  },
  validators: {
    onSubmit: ({ value }) =>
      value.name.length === 0 ? "Name is required" : undefined,
  },
});

const emit = defineEmits<{
  (e: "deleted"): void;
}>();

const open = (
  openViewId: string,
  openCanDeleteView: boolean,
  openIsDefaultView: boolean,
) => {
  viewId.value = openViewId;
  canDeleteView.value = openCanDeleteView;
  isDefaultView.value = openIsDefaultView;
  modalRef.value?.open();
};
const close = () => {
  viewId.value = "";
  canDeleteView.value = false;
  isDefaultView.value = false;
  modalRef.value?.close();
};
defineExpose({ open, close });
</script>
