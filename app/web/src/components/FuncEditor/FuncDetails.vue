<template>
  <EmptyStateCard
    v-if="!funcId"
    iconName="funcs"
    primaryText="No Function Selected"
    secondaryText="Select a function from the list on the left panel to view its details here."
  />
  <LoadingMessage
    v-else-if="
      (loadFuncDetailsReqStatus.isPending && !storeFuncDetails) ||
      !loadFuncDetailsReqStatus.isRequested
    "
  >
    Loading function "{{ selectedFuncSummary?.name }}"
  </LoadingMessage>
  <div
    v-else-if="selectedFuncId && editingFunc"
    :class="clsx('h-full w-full flex flex-col overflow-hidden')"
  >
    <div class="flex flex-col">
      <ErrorMessage
        v-if="editingFunc?.associations?.type === 'action'"
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
      <ErrorMessage
        v-if="isConnectedToOtherAssetTypes"
        icon="alert-triangle"
        variant="block"
        tone="warning"
      >
        This function is connected to other
        {{
          editingFunc?.associations &&
          editingFunc?.associations?.type === "attribute"
            ? "attributes"
            : "assets"
        }}.
      </ErrorMessage>

      <SidebarSubpanelTitle
        icon="func"
        :label="selectedFuncSummary?.name"
        variant="subtitle"
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
          :disabled="
            !(
              funcStore.selectedFuncDetails &&
              funcStore.selectedFuncDetails?.associations?.type !==
                'authentication'
            )
          "
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
          :disabled="!schemaVariantId"
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
              v-if="editingFunc.associations?.type === 'action'"
              ref="detachRef"
              v-model="editingFunc.associations"
              :requestStatus="updateFuncReqStatus"
              :schemaVariantId="schemaVariantId"
              @change="updateFunc"
            />
            <AuthenticationDetails
              v-if="editingFunc.associations?.type === 'authentication'"
              ref="detachRef"
              v-model="editingFunc.associations"
              :schemaVariantId="schemaVariantId"
              @change="updateFunc"
            />
            <CodeGenerationDetails
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'codeGeneration'
              "
              ref="detachRef"
              v-model="editingFunc.associations"
              :schemaVariantId="schemaVariantId"
              @change="updateFunc"
            />
            <QualificationDetails
              v-if="
                editingFunc.associations &&
                editingFunc.associations.type === 'qualification'
              "
              ref="detachRef"
              v-model="editingFunc.associations"
              :schemaVariantId="schemaVariantId"
              @change="updateFunc"
            />

            <TreeNode
              v-if="editingFunc.kind === FuncKind.Attribute"
              label="Arguments"
              defaultOpen
              enableGroupToggle
              alwaysShowArrow
              indentationSize="none"
              leftBorderSize="none"
              labelClasses="border-b border-neutral-200 dark:border-neutral-600"
              childrenContainerClasses="border-b border-neutral-200 dark:border-neutral-600"
            >
              <FuncArguments
                v-if="
                  editingFunc.associations &&
                  editingFunc.associations.type === 'attribute'
                "
                :funcId="editingFunc.id"
              />
            </TreeNode>

            <TreeNode
              v-if="editingFunc.kind === FuncKind.Attribute && schemaVariantId"
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
                v-if="
                  editingFunc.associations &&
                  editingFunc.associations.type === 'attribute'
                "
                ref="detachRef"
                v-model="editingFunc.associations"
                :schemaVariantId="schemaVariantId"
                @change="updateFunc"
              />
            </TreeNode>
          </div>
        </TabGroupItem>

        <TabGroupItem
          v-if="editingFunc.kind === FuncKind.Attribute"
          label="Bindings"
          slug="bindings"
        >
          <AttributeBindings
            v-if="
              editingFunc.associations &&
              editingFunc.associations.type === 'attribute'
            "
            ref="detachRef"
            v-model="editingFunc.associations"
            class="border-t border-neutral-200 dark:border-neutral-600"
            @change="updateFunc"
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
  LoadingMessage,
  Stack,
  TabGroup,
  TabGroupItem,
  TreeNode,
  VormInput,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { FuncKind, FuncId } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import AuthenticationDetails from "@/components/FuncEditor/AuthenticationDetails.vue";
import FuncArguments from "./FuncArguments.vue";
import ActionDetails from "./ActionDetails.vue";
import AttributeBindings from "./AttributeBindings.vue";
import CodeGenerationDetails from "./CodeGenerationDetails.vue";
import QualificationDetails from "./QualificationDetails.vue";
import FuncTest from "./FuncTest.vue";
import EmptyStateCard from "../EmptyStateCard.vue";
import IconButton from "../IconButton.vue";
import SidebarSubpanelTitle from "../SidebarSubpanelTitle.vue";

const props = defineProps<{
  funcId?: FuncId;
  schemaVariantId?: string;
  allowTestPanel?: boolean;
}>();

const funcDetailsTabGroupRef = ref();

const funcStore = useFuncStore();
const assetStore = useAssetStore();

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

const loadFuncDetailsReqStatus = funcStore.getRequestStatus(
  "FETCH_FUNC",
  funcId,
);
const updateFuncReqStatus = funcStore.getRequestStatus("UPDATE_FUNC", funcId);
const { selectedFuncId, selectedFuncSummary } = storeToRefs(funcStore);

const storeFuncDetails = computed(() => funcStore.selectedFuncDetails);
const editingFunc = ref(_.cloneDeep(storeFuncDetails.value));

function resetEditingFunc() {
  editingFunc.value = _.cloneDeep(storeFuncDetails.value);
}

// when the func details finish loading, we copy into our local draft
watch([loadFuncDetailsReqStatus, updateFuncReqStatus], () => {
  resetEditingFunc();
});

watch(
  () => funcStore.selectedFuncId,
  () => {
    if (funcStore.selectedFuncId) {
      funcStore.FETCH_FUNC(funcStore.selectedFuncId);
    }

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
  if (
    !editingFunc.value ||
    _.isEqual(editingFunc.value, storeFuncDetails.value)
  )
    return;
  funcStore.UPDATE_FUNC(editingFunc.value);
};

const isConnectedToOtherAssetTypes = computed(() => {
  if (editingFunc?.value && editingFunc?.value?.associations) {
    const associations = editingFunc.value.associations;
    switch (associations.type) {
      case "codeGeneration":
      case "qualification":
        return (
          associations.schemaVariantIds.length > 1 ||
          associations.componentIds.length > 1
        );
      case "action":
        return associations.schemaVariantIds.length > 1;
      case "attribute":
        return associations.prototypes.length > 1;
      default:
        return false;
    }
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
    const associations = await detachRef.value.detachFunc();
    if (assetStore.selectedAssetId)
      assetStore.LOAD_ASSET(assetStore.selectedAssetId); // reloads the fn list
    if (funcStore.selectedFuncId)
      assetStore.removeFuncSelection(funcStore.selectedFuncId);
    if (funcStore.selectedFuncId && assetStore.selectedAssetId)
      assetStore.closeFunc(
        assetStore.selectedAssetId,
        funcStore.selectedFuncId,
      );
    funcStore.selectedFuncId = undefined; // brings you back to the asset detail

    if (associations && editingFunc.value) {
      isDetaching.value = true;
      await funcStore.UPDATE_FUNC({
        ...editingFunc.value,
        associations,
      });
      isDetaching.value = false;
    }
  }
};

const isDeleting = ref(false);
const deleteFunc = async () => {
  if (!funcId.value) return;
  await funcStore.DELETE_FUNC(funcId.value);
};

const hasAssociations = computed(() => {
  if (editingFunc?.value) {
    return (
      editingFunc.value.associations === undefined &&
      !editingFunc.value.isBuiltin
    );
  }
  return false;
});

// The parent component can allow the test panel to be enabled, but we need to dynamically enable
// it based on the func kind.
const enableTestPanel = computed((): boolean => {
  return (
    props.allowTestPanel &&
    (funcStore.selectedFuncDetails?.associations?.type === "action" ||
      funcStore.selectedFuncDetails?.associations?.type === "attribute" ||
      funcStore.selectedFuncDetails?.associations?.type === "codeGeneration" ||
      funcStore.selectedFuncDetails?.associations?.type === "qualification")
  );
});
</script>
