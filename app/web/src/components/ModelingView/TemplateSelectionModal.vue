<template>
  <Modal ref="modalRef" title="Extract Template" size="xl" noAutoFocus>
    <div class="flex flex-col gap-sm max-h-[70vh]">
      <div class="flex-none">
        This will create a new template function based on
        {{
          selectedComponents.length === 1
            ? "this component"
            : `the following ${selectedComponents.length} components`
        }}. The new function will be attached to a new asset.
      </div>

      <div class="flex flex-col flex-1 gap-xs overflow-y-auto">
        <ComponentCard
          v-for="component in selectedComponents"
          :key="component.def.id"
          :component="component"
        />
      </div>

      <div class="flex flex-col flex-none">
        <VormInput
          compact
          compactWide
          required
          label="Asset Color"
          type="container"
        >
          <ColorPicker
            id="asset-color"
            v-model="assetColor"
            insideModal
            @change="null"
          />
        </VormInput>
        <VormInput
          v-model="assetName"
          compact
          compactWide
          label="Asset Name"
          required
          placeholder="A name for your generated Template Asset."
        />
        <VormInput
          v-model="funcName"
          compact
          compactWide
          label="Function Name"
          required
          placeholder="A name for your generated Template Function."
        />
      </div>

      <div class="flex flex-row gap-sm flex-none">
        <VButton label="Cancel" tone="shade" variant="ghost" @click="close" />
        <VButton
          label="Create Template"
          icon="plus"
          tone="action"
          class="flex-grow"
          :disabled="!readyToSubmit"
          @click="onCreateTemplate"
        />
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {
  ColorPicker,
  Modal,
  useModal,
  VButton,
  VormInput,
} from "@si/vue-lib/design-system";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { useComponentsStore } from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import ComponentCard from "../ComponentCard.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramViewData,
} from "../ModelingDiagram/diagram_types";

const componentsStore = useComponentsStore();
const viewsStore = useViewsStore();
const featureFlagsStore = useFeatureFlagsStore();

const { selectedComponents, restorableSelectedComponents } =
  storeToRefs(viewsStore);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const randomDefaultColor = () => {
  return `#${_.sample([
    "ff0000",
    "00ff00",
    "0000ff",
    "ffff00",
    "ff00ff",
    "00ffff",
  ])}`;
};

const assetColor = ref(randomDefaultColor());
const assetName = ref("");
const funcName = ref("");

const validSelectedComponents = computed(
  () =>
    !selectedComponents.value.some((c) => c instanceof DiagramViewData) && // no views
    restorableSelectedComponents.value.length === 0, // no components marked for deletion
);

const readyToSubmit = computed(() =>
  Boolean(assetName.value && funcName.value),
);

const open = () => {
  // only open the modal if the selected components are valid for template generation
  // and the feature flag is enabled!
  if (
    !validSelectedComponents.value ||
    !featureFlagsStore.TEMPLATE_MGMT_FUNC_GENERATION
  ) {
    return;
  }

  assetColor.value = randomDefaultColor();
  assetName.value = "";
  funcName.value = "";
  openModal();
};

const modelingEventBus = componentsStore.eventBus;
onMounted(() => {
  modelingEventBus.on("templateFromSelection", open);
});
onBeforeUnmount(() => {
  modelingEventBus.off("templateFromSelection", open);
});

const onCreateTemplate = () => {
  if (!readyToSubmit.value || !validSelectedComponents.value) return;

  const templateData = {
    assetColor: assetColor.value,
    assetName: assetName.value,
    funcName: funcName.value,
    components: selectedComponents.value as Array<
      DiagramNodeData | DiagramGroupData
    >,
  };

  componentsStore.CREATE_TEMPLATE_FUNC_FROM_COMPONENTS(templateData);

  close();
};

defineExpose({ open, close });
</script>
