<template>
  <!-- NOTE: the Modal CSS for height in "max" doesn't work as we might expect -->
  <Modal ref="modalRef" size="lg" title="Edit View">
    <div class="flex flex-col">
      <nameForm.Field name="name">
        <template #default="{ field }">
          <div
            v-if="field.state.meta.errors.length > 0"
            :class="clsx('text-sm mb-xs', themeClasses('text-destructive-600', 'text-destructive-200'))"
          >
            {{ field.state.meta.errors[0] }}
          </div>
        </template>
      </nameForm.Field>

      <label class="flex flex-row items-center relative">
        <span>View Name*</span>
        <nameForm.Field
          name="name"
          :validators="{
            onChange: viewNameValidator,
            onBlur: viewNameValidator,
          }"
        >
          <template #default="{ field }">
            <input
              :class="
                clsx(
                  'block w-72 ml-auto border',
                  field.state.meta.errors.length > 0
                    ? themeClasses(
                        'text-black bg-white border-destructive-600 disabled:bg-neutral-100',
                        'text-white bg-black border-destructive-400 disabled:bg-neutral-900',
                      )
                    : themeClasses(
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
        <Icon v-if="wForm.bifrosting.value" class="absolute right-2xs" name="loader" size="sm" tone="action" />
      </label>
    </div>

    <div class="flex gap-sm mt-sm">
      <NewButton
        v-tooltip="tooltipText"
        label="Delete View"
        :disabled="canDeleteView !== 'yes'"
        :loading="canDeleteView === 'loading'"
        @click="deleteView"
      />
      <div class="flex gap-sm ml-auto">
        <NewButton label="Cancel" pill="ESC" @click="() => modalRef?.close()" />
        <nameForm.Field name="name">
          <template #default="{ field }">
            <NewButton
              label="Done"
              tone="action"
              :loading="wForm.bifrosting.value"
              :disabled="wForm.bifrosting.value || field.state.meta.errors.length > 0 || field.state.value === ''"
              @click="() => nameForm.handleSubmit()"
            />
          </template>
        </nameForm.Field>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import { Modal, Icon, themeClasses, NewButton } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import clsx from "clsx";
import { useRoute } from "vue-router";
import { useQuery } from "@tanstack/vue-query";
import { ViewId } from "@/api/sdf/dal/views";
import { ComponentInList, EntityKind, View } from "@/workers/types/entity_kind_types";
import { Listable } from "@/workers/types/dbinterface";
import { bifrostList, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { useWatchedForm } from "./logic_composables/watched_form";
import { useApi, routes } from "./api_composables";

const props = defineProps<{
  views: View[] | undefined;
}>();

const viewId = ref<ViewId>("");
const isDefaultView = ref<boolean>(false);

const tooltipText = computed(() => {
  if (isDefaultView.value) return "Cannot delete the default view.";
  if (canDeleteView.value === "yes") return undefined;
  if (canDeleteView.value === "loading") return "Determining if the view can be deleted...";
  return "Views containing one or more components cannot be deleted. To delete a view, first remove all components from it.";
});

const key = useMakeKey();
const args = useMakeArgs();
const keyViewId = computed(() => viewId.value);
const componentListRaw = useQuery<ComponentInList[]>({
  // NOTE(nick): @britmyerss saved my life here. You need the "() =>" to evaluate this as a function
  // The first PR would've done this, but I ripped it out. NOW, IT WORKS. YES.
  enabled: () => keyViewId.value !== "",
  queryKey: key(EntityKind.ViewComponentList, keyViewId),
  queryFn: async () => {
    const arg = args<Listable>(EntityKind.ViewComponentList, keyViewId.value);
    const list = await bifrostList<ComponentInList[]>(arg);
    return list ?? [];
  },
});
const canDeleteView = computed(() => {
  if (isDefaultView.value) return "no";
  if (!componentListRaw.data.value) return "loading";
  if (componentListRaw.data.value.length < 1) return "yes";
  return "no";
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

const existingViewNames = computed(
  () => props.views?.map((view) => view.name).filter((name) => name !== nameOnOpen.value) ?? [],
);
const viewNameValidator = ({ value }: { value: string }) => {
  if (value.trim().length === 0) {
    return "Name is required";
  } else if (existingViewNames.value.includes(value)) {
    return "That view name is already in use";
  }
  return undefined;
};

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
      // manually triggering the watcher to close the loop
      nameOnOpen.value = value.name;
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
    onSubmit: ({ value }) => viewNameValidator({ value: value.name }),
  },
  watchFn: () => nameOnOpen.value,
});

const emit = defineEmits<{
  (e: "deleted"): void;
}>();

const nameOnOpen = ref("");
const open = (openViewId: string, openViewName: string, openIsDefaultView: boolean) => {
  viewId.value = openViewId;
  isDefaultView.value = openIsDefaultView;
  modalRef.value?.open();
  wForm.reset(nameForm);
  nameOnOpen.value = openViewName;
  nameForm.setFieldValue("name", openViewName);
};
const close = () => {
  viewId.value = "";
  isDefaultView.value = false;
  modalRef.value?.close();
};
defineExpose({ open, close, isOpen: modalRef.value?.isOpen });
</script>
