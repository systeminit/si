<template>
  <!-- NOTE: the Modal CSS for height in "max" doesn't work as we might expect -->
  <Modal ref="modalRef" size="lg" title="Create View">
    <form @submit.prevent="nameForm.handleSubmit()">
      <nameForm.Field name="name">
        <template #default="{ field }">
          <div
            v-if="field.state.meta.errors.length > 0"
            :class="
              clsx(
                'text-sm mb-xs',
                themeClasses('text-destructive-600', 'text-destructive-200'),
              )
            "
          >
            {{ field.state.meta.errors[0] }}
          </div>
        </template>
      </nameForm.Field>

      <label class="flex flex-row items-center relative">
        <span>View Name</span>
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
        <Icon
          v-if="wForm.bifrosting.value"
          class="absolute right-2xs"
          name="loader"
          size="sm"
          tone="action"
        />
      </label>
    </form>

    <div class="flex gap-sm mt-sm">
      <NewButton label="Cancel" @click="() => modalRef?.close()" />
      <nameForm.Field name="name">
        <template #default="{ field }">
          <NewButton
            class="flex-grow"
            icon="plus"
            label="Create"
            tone="action"
            :loading="wForm.bifrosting.value"
            :disabled="
              wForm.bifrosting.value ||
              field.state.meta.errors.length > 0 ||
              field.state.value === ''
            "
            @click="() => nameForm.handleSubmit()"
          />
        </template>
      </nameForm.Field>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import {
  Modal,
  Icon,
  NewButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
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

const existingViewNames = computed(
  () => props.views?.map((view) => view.name) ?? [],
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
    const call = api.endpoint<{ id: string }>(routes.CreateView);
    const { req, newChangeSetId } = await call.post<{ name: string }>({
      name: value.name,
    });
    if (api.ok(req)) {
      modalRef.value?.close();
      if (newChangeSetId) {
        api.navigateToNewChangeSet(
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
  watchFn: () => props.views?.length ?? 0,
});

const open = () => {
  modalRef.value?.open();
  wForm.reset(nameForm);
  nameForm.state.values.name = "";
};
defineExpose({ open, isOpen: modalRef.value?.isOpen });
</script>
