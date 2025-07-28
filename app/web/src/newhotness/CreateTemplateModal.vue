<template>
  <ConfirmModal
    ref="modalRef"
    title="Create template"
    confirmLabel="Create"
    @confirm="confirm"
    @keydown.enter="confirm"
  >
    <ErrorMessage v-if="requestError">{{ requestError }}</ErrorMessage>
    <div>
      Creates a function that, when run, adds the selected components to your
      application. It will be saved as a new asset. Name the asset and label the
      function below.
    </div>
    <div
      v-for="(field, idx) in formStructure"
      :key="idx"
      class="flex flex-row justify-between text-sm"
    >
      <span class="mt-xs"> {{ field.title }} * </span>
      <div class="w-3/5 flex flex-col gap-2xs">
        <input
          v-model="field.ref.value"
          :class="
            clsx(
              'h-lg p-xs text-sm border font-mono cursor-text',
              'focus:outline-none focus:ring-0 focus:z-10',
              themeClasses(
                'text-shade-100 bg-white border-neutral-400 focus:border-action-500',
                'text-shade-0 bg-black border-neutral-600 focus:border-action-300',
              ),
            )
          "
        />
        <span class="text-xs text-neutral-400">
          {{ field.hint }}
        </span>
      </div>
    </div>
  </ConfirmModal>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { ErrorMessage, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useRoute } from "vue-router";
import { ComponentId } from "@/api/sdf/dal/component";
import { routes, useApi } from "@/newhotness/api_composables";
import ConfirmModal from "./layout_components/ConfirmModal.vue";

const route = useRoute();

const modalRef = ref<InstanceType<typeof ConfirmModal>>();
const requestError = ref<string | undefined>();

// FORM FIELDS
const assetName = ref<string>("");
const funcName = ref<string>("");
const category = ref<string>("");

const formStructure = [
  {
    title: "Template name",
    hint: "This name helps you identify and reuse the template later",
    ref: assetName,
  },
  {
    title: "Function label",
    hint: "Describe what the template does (e.g., Create VPC setup)",
    ref: funcName,
  },
  {
    title: "Category label",
    hint: "Categorize your template to find it faster when adding components",
    ref: category,
  },
];

// Static values
const viewIdRef = ref<string | undefined>();
const componentIdsRef = ref<ComponentId[] | undefined>();

function open(componentIds: ComponentId[], viewId: string) {
  if (!componentIds.length) return;

  componentIdsRef.value = componentIds;
  viewIdRef.value = viewId;

  requestError.value = undefined;
  assetName.value = "";
  funcName.value = "";
  category.value = "Templates";

  modalRef.value?.open();
}
function close() {
  modalRef.value?.close();
}

const createTemplateApi = useApi();
async function confirm() {
  const viewId = viewIdRef.value;
  const componentIds = componentIdsRef.value;
  if (!viewId || !componentIds) {
    return;
  }
  const color = "#AAAAAA"; // Hardcoded color since it does not matter for the new UI. We'll remove this eventually

  // If any of the form fields are empty, do not confirm
  if (
    [assetName.value, funcName.value, category.value].find(
      (val) => val === "",
    ) !== undefined
  ) {
    requestError.value = "All fields are required";
    return;
  }

  requestError.value = undefined;
  const call = createTemplateApi.endpoint(routes.CreateTemplate, { viewId });
  const { req, newChangeSetId, errorMessage } = await call.post({
    componentIds,
    assetName: assetName.value,
    funcName: funcName.value,
    category: category.value,
    color,
  });

  if (errorMessage) {
    requestError.value = errorMessage;
    return;
  }

  if (createTemplateApi.ok(req)) {
    if (newChangeSetId) {
      createTemplateApi.navigateToNewChangeSet(
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

  close();
}

defineExpose({ open, close });
</script>
