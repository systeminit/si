<template>
  <!-- NOTE: the Modal CSS for height in "max" doesn't work as we might expect -->
  <Modal
    ref="modalRef"
    size="lg"
    title="Create View"
    type="save"
    saveLabel="Create"
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
            @input="(e) => field.handleChange((e.target as HTMLInputElement).value)"
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
  </Modal>
</template>

<script setup lang="ts">
import { Modal, Icon, themeClasses } from "@si/vue-lib/design-system";
import { computed, ref, watch } from "vue";
import { useRoute } from "vue-router";
import clsx from "clsx";
import { View } from "@/workers/types/entity_kind_types";
import { useApi, routes } from "./api_composables";
import { useWatchedForm } from "./logic_composables/watched_form";

const props = defineProps<{
  views: View[] | undefined;
}>();

const modalRef = ref<InstanceType<typeof Modal>>();

const api = useApi();

const route = useRoute();

const wForm = useWatchedForm<{ name: string }>("view.add");
const formData = computed<{ name: string }>(() => {
  return { name: "" };
});

const nameForm = wForm.newForm({
  data: formData,
  onSubmit: async ({ value }) => {
    const call = api.endpoint<{ id: string }>(routes.CreateView);
    const { req, newChangeSetId } = await call.post<{ name: string }>({
      name: value.name,
    });
    if (api.ok(req)) {
      // right now, we don't want to push people to the new view
      // because they won't see components, and won't be able to add any to the view
      if (newChangeSetId) {
        api.navigateToNewChangeSet(
          {
            name: "new-hotness-view",
            params: {
              workspacePk: route.params.workspacePk,
              changeSetId: newChangeSetId,
            },
          },
          newChangeSetId,
        );
      }
      watch(
        () => wForm.bifrosting,
        () => {
          modalRef.value?.close();
        },
        { once: true },
      );
    }
  },
  validators: {
    onSubmit: ({ value }) =>
      value.name.length === 0 ? "Name is required" : undefined,
  },
  watchFn: () => props.views?.length ?? 0,
});

const open = () => {
  modalRef.value?.open();
};
defineExpose({ open });
</script>
