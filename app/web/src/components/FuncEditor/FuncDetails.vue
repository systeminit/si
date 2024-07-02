<template>
  <EmptyStateCard
    v-if="!funcId"
    iconName="funcs"
    primaryText="No Function Selected"
    secondaryText="Select a function from the list on the left panel to view its details here."
  />
  <div
    v-else-if="selectedFuncSummary && editingFunc"
    :class="clsx('h-full w-full flex flex-col overflow-hidden')"
  >
    <div class="flex flex-col">
      <SidebarSubpanelTitle
        icon="func"
        variant="subtitle"
        :label="selectedFuncSummary.name"
      >
        <template v-if="ffStore.IMMUTABLE_SCHEMA_VARIANTS">
          <EditingPill
            v-if="!selectedFuncSummary.isLocked"
            color="#666"
          ></EditingPill>
          <IconButton
            v-else
            class="hover:scale-125"
            variant="simple"
            icon="sliders-vertical"
            tooltip="Edit"
            tooltipPlacement="top"
          />
        </template>
      </SidebarSubpanelTitle>
      <ErrorMessage
        v-if="isConnectedToOtherAssetTypes"
        icon="alert-triangle"
        variant="block"
        tone="warning"
      >
        This function is connected to other
        {{
          selectedFuncSummary.kind === FuncKind.Attribute
            ? "attributes"
            : "assets"
        }}.
      </ErrorMessage>
      <ErrorMessage
        v-if="selectedFuncSummary.kind === FuncKind.Action"
        icon="alert-triangle"
        tone="warning"
        variant="block"
        >Executing this will run on all attached components and may affect your
        real-world resources!
      </ErrorMessage>
      <ErrorMessage
        v-if="execFuncReqStatus.isError"
        :requestStatus="execFuncReqStatus"
        variant="block"
      />
      <div
        class="flex flex-row gap-2xs items-center justify-evenly py-xs border-b border-neutral-200 dark:border-neutral-600"
      >
        <IconButton
          icon="save"
          loadingIcon="loader"
          iconTone="success"
          loadingTooltip="Executing..."
          tooltip="Execute"
          tooltipPlacement="top"
          :requestStatus="execFuncReqStatus"
          :disabled="!(selectedFuncSummary.kind !== FuncKind.Authentication)"
          @click="execFunc"
        />
        <IconButton
          tooltip="Test"
          tooltipPlacement="top"
          icon="test-tube"
          iconTone="action"
          :disabled="!enableTestPanel"
          @click="funcDetailsTabGroupRef.selectTab('test')"
        />
        <IconButton
          :loading="isDetaching"
          iconTone="warning"
          icon="unlink"
          tooltip="Detach"
          tooltipPlacement="top"
          loadingTooltip="Detaching..."
          @click="detachFunc"
        />
        <IconButton
          :loading="isDeleting"
          iconTone="destructive"
          :disabled="hasAssociations"
          icon="trash"
          tooltip="Delete"
          tooltipPlacement="top"
          loadingTooltip="Deleting..."
          @click="deleteFunc"
        />
      </div>
    </div>
    <div class="flex-grow relative">
      <TabGroup
        ref="funcDetailsTabGroupRef"
        growTabsToFillWidth
        variant="fullsize"
      >
        <TabGroupItem label="Properties" slug="properties">
          <div
            class="flex flex-col absolute inset-0 overflow-y-auto overflow-x-hidden border-t border-neutral-200 dark:border-neutral-600"
          >
            <TreeNode
              label="Attributes"
              defaultOpen
              enableGroupToggle
              alwaysShowArrow
              noIndentationOrLeftBorder
              labelClasses="border-b border-neutral-200 dark:border-neutral-600"
              childrenContainerClasses="border-b border-neutral-200 dark:border-neutral-600"
            >
              <Stack class="p-xs" spacing="none">
                <VormInput
                  v-model="editingFunc.name"
                  label="Name"
                  required
                  compact
                  @blur="updateFunc"
                />
                <VormInput
                  v-model="editingFunc.displayName"
                  label="Display Name"
                  required
                  compact
                  @blur="updateFunc"
                />
                <VormInput
                  v-model="editingFunc.description"
                  type="textarea"
                  compact
                  label="Description"
                  @blur="updateFunc"
                />
              </Stack>
            </TreeNode>

            <ActionDetails
              v-if="selectedFuncSummary.kind === FuncKind.Action"
              ref="detachRef"
              :funcId="selectedFuncSummary.funcId"
              :schemaVariantId="$props.schemaVariantId"
            />
            <AuthenticationDetails
              v-if="selectedFuncSummary.kind === FuncKind.Authentication"
              ref="detachRef"
              :funcId="selectedFuncSummary.funcId"
              :schemaVariantId="$props.schemaVariantId"
            />
            <CodeGenerationDetails
              v-if="selectedFuncSummary.kind === FuncKind.CodeGeneration"
              ref="detachRef"
              :funcId="selectedFuncSummary.funcId"
              :schemaVariantId="$props.schemaVariantId"
            />
            <QualificationDetails
              v-if="selectedFuncSummary.kind === FuncKind.Qualification"
              ref="detachRef"
              :funcId="selectedFuncSummary.funcId"
              :schemaVariantId="$props.schemaVariantId"
            />

            <TreeNode
              v-if="selectedFuncSummary?.kind === FuncKind.Attribute"
              label="Arguments"
              defaultOpen
              enableGroupToggle
              alwaysShowArrow
              indentationSize="none"
              leftBorderSize="none"
              labelClasses="border-b border-neutral-200 dark:border-neutral-600"
              childrenContainerClasses="border-b border-neutral-200 dark:border-neutral-600"
            >
              <FuncArguments :funcId="editingFunc.funcId" />
            </TreeNode>

            <TreeNode
              v-if="
                selectedFuncSummary?.kind === FuncKind.Attribute &&
                $props.schemaVariantId
              "
              label="Bindings"
              defaultOpen
              enableGroupToggle
              alwaysShowArrow
              indentationSize="none"
              leftBorderSize="none"
              labelClasses="border-b border-neutral-200 dark:border-neutral-600"
              childrenContainerClasses="border-b border-neutral-200 dark:border-neutral-600"
            >
              <AttributeBindings
                ref="detachRef"
                :schemaVariantId="$props.schemaVariantId"
                :funcId="selectedFuncSummary.funcId"
              />
            </TreeNode>
          </div>
        </TabGroupItem>

        <TabGroupItem
          v-if="selectedFuncSummary?.kind === FuncKind.Attribute"
          label="Bindings"
          slug="bindings"
        >
          <AttributeBindings
            ref="detachRef"
            :funcId="selectedFuncSummary.funcId"
            class="border-t border-neutral-200 dark:border-neutral-600"
          />
        </TabGroupItem>

        <TabGroupItem v-if="enableTestPanel" label="Test" slug="test">
          <FuncTest />
        </TabGroupItem>
      </TabGroup>
    </div>
  </div>
  <div
    v-else
    class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
  >
    Function "{{ funcId }}" does not exist!
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import {
  ErrorMessage,
  Stack,
  TabGroup,
  TabGroupItem,
  TreeNode,
  VormInput,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import {
  FuncKind,
  FuncId,
  FuncBinding,
  FuncBindingKind,
} from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import AuthenticationDetails from "@/components/FuncEditor/AuthenticationDetails.vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import FuncArguments from "./FuncArguments.vue";
import ActionDetails from "./ActionDetails.vue";
import AttributeBindings from "./AttributeBindings.vue";
import CodeGenerationDetails from "./CodeGenerationDetails.vue";
import QualificationDetails from "./QualificationDetails.vue";
import FuncTest from "./FuncTest.vue";
import EmptyStateCard from "../EmptyStateCard.vue";
import IconButton from "../IconButton.vue";
import SidebarSubpanelTitle from "../SidebarSubpanelTitle.vue";
import EditingPill from "../EditingPill.vue";

const props = defineProps<{
  funcId: FuncId;
  schemaVariantId: string;
  allowTestPanel?: boolean;
}>();

const funcDetailsTabGroupRef = ref();

const funcStore = useFuncStore();
const assetStore = useAssetStore();
const ffStore = useFeatureFlagsStore();

const emit = defineEmits<{
  (e: "expandPanel"): void;
}>();

type DetachType =
  | InstanceType<typeof ActionDetails>
  | InstanceType<typeof AttributeBindings>
  | InstanceType<typeof CodeGenerationDetails>
  | InstanceType<typeof QualificationDetails>;

const detachRef = ref<DetachType>();
const funcId = computed(() => props.funcId);

const updateFuncReqStatus = funcStore.getRequestStatus("UPDATE_FUNC", funcId);
const { selectedFuncSummary } = storeToRefs(funcStore);

const editingFunc = ref(_.cloneDeep(selectedFuncSummary.value));

function resetEditingFunc() {
  editingFunc.value = _.cloneDeep(selectedFuncSummary.value);
}

// when the func details finish loading, we copy into our local draft
watch([updateFuncReqStatus], () => {
  resetEditingFunc();
});

watch(
  () => funcStore.selectedFuncId,
  () => {
    if (
      funcDetailsTabGroupRef.value &&
      funcDetailsTabGroupRef.value.tabExists("properties")
    ) {
      funcDetailsTabGroupRef.value.selectTab("properties");
    }
  },
  { immediate: true },
);

const updateFunc = () => {
  if (editingFunc.value) funcStore.UPDATE_FUNC(editingFunc.value);
};

// THIS FEELS UNNECESSARY NOW
const isConnectedToOtherAssetTypes = computed<boolean>(() => {
  if (
    selectedFuncSummary.value &&
    selectedFuncSummary.value.bindings.length > 0
  ) {
    return selectedFuncSummary.value.bindings
      .map((b: FuncBinding) => {
        switch (b.bindingKind) {
          case FuncBindingKind.Qualification:
          case FuncBindingKind.CodeGeneration:
            return !!b.schemaVariantId;
          case FuncBindingKind.Action:
            return !!b.schemaVariantId;
          case FuncBindingKind.Attribute:
            return !!b.attributePrototypeId;
          default:
            return false;
        }
      })
      .some(Boolean);
  }
  return false;
});

const execFuncReqStatus = funcStore.getRequestStatus(
  "SAVE_AND_EXEC_FUNC",
  funcId,
);
const execFunc = () => {
  if (!funcId.value) return;
  funcStore.SAVE_AND_EXEC_FUNC(funcId.value);
};

const isDetaching = ref(false);
const detachFunc = async () => {
  if (detachRef.value && "detachFunc" in detachRef.value) {
    detachRef.value.detachFunc();
    if (assetStore.selectedVariantId)
      assetStore.LOAD_SCHEMA_VARIANT(assetStore.selectedVariantId);
    if (funcStore.selectedFuncId) assetStore.setFuncSelection(undefined);
  }
};

const isDeleting = ref(false);
const deleteFunc = async () => {
  if (!funcId.value) return;
  await funcStore.DELETE_FUNC(funcId.value);
};

const hasAssociations = computed(() => {
  if (selectedFuncSummary.value?.bindings.length === 0) return true;
  return false;
});

// The parent component can allow the test panel to be enabled, but we need to dynamically enable
// it based on the func kind.
const enableTestPanel = computed(() => {
  return (
    props.allowTestPanel &&
    selectedFuncSummary.value &&
    selectedFuncSummary.value.bindings
      .map((b) => b.bindingKind)
      .filter((kind) =>
        [
          FuncBindingKind.Action,
          FuncBindingKind.Attribute,
          FuncBindingKind.CodeGeneration,
          FuncBindingKind.Qualification,
        ].includes(kind),
      ).length > 0
  );
});
</script>
