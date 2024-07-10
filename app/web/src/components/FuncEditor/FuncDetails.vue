<template>
  <EmptyStateCard
    v-if="!funcId"
    iconName="funcs"
    primaryText="No Function Selected"
    secondaryText="Select a function from the list on the left panel to view its details here."
  />
  <div
    v-else-if="editingFunc"
    :class="clsx('h-full w-full flex flex-col overflow-hidden')"
  >
    <div class="flex flex-col">
      <SidebarSubpanelTitle
        :label="editingFunc.name"
        icon="func"
        variant="subtitle"
      >
        <div class="flex flex-row">
          <EditingPill v-if="!editingFunc.isLocked" color="#666"></EditingPill>
          <IconButton
            v-if="editingFunc.isLocked"
            :class="clsx(!unlocking && 'hover:scale-125')"
            :loading="unlocking"
            icon="sliders-vertical"
            tooltip="Edit"
            tooltipPlacement="top"
            variant="simple"
            @click="unlock"
          />
          <IconButton
            v-else
            :class="clsx('mx-xs', !isDeleting && 'hover:scale-125')"
            :loading="isDeleting"
            icon="trash"
            iconTone="destructive"
            loadingTooltip="Deleting..."
            size="sm"
            tooltip="Delete"
            tooltipPlacement="top"
            @click="deleteFunc"
          />
        </div>
      </SidebarSubpanelTitle>
      <ErrorMessage
        v-if="isConnectedToOtherSchemas"
        icon="alert-triangle"
        tone="warning"
        variant="block"
      >
        This function is connected to other
        {{ editingFunc.kind === FuncKind.Attribute ? "attributes" : "assets" }}.
      </ErrorMessage>
      <ErrorMessage
        v-if="editingFunc.kind === FuncKind.Action"
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
          :disabled="
            !(editingFunc.kind !== FuncKind.Authentication) ||
            editingFunc.isLocked
          "
          :requestStatus="execFuncReqStatus"
          icon="save"
          iconTone="success"
          loadingIcon="loader"
          loadingTooltip="Executing..."
          tooltip="Execute"
          tooltipPlacement="top"
          @click="execFunc"
        />
        <IconButton
          :disabled="!enableTestPanel"
          icon="test-tube"
          iconTone="action"
          tooltip="Test"
          tooltipPlacement="top"
          @click="funcDetailsTabGroupRef.selectTab('test')"
        />
        <IconButton
          :disabled="schemaVariant?.isLocked"
          :loading="isDetaching"
          icon="unlink"
          iconTone="warning"
          loadingTooltip="Detaching..."
          tooltip="Detach"
          tooltipPlacement="top"
          @click="detachFunc"
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
              alwaysShowArrow
              childrenContainerClasses="border-b border-neutral-200 dark:border-neutral-600"
              defaultOpen
              enableGroupToggle
              label="Attributes"
              labelClasses="border-b border-neutral-200 dark:border-neutral-600"
              noIndentationOrLeftBorder
            >
              <Stack class="p-xs" spacing="none">
                <VormInput
                  v-model="editingFunc.name"
                  :disabled="editingFunc.isLocked"
                  compact
                  label="Name"
                  required
                  @blur="updateFunc"
                />
                <VormInput
                  v-model="editingFunc.displayName"
                  :disabled="editingFunc.isLocked"
                  compact
                  label="Display Name"
                  required
                  @blur="updateFunc"
                />
                <VormInput
                  v-model="editingFunc.description"
                  :disabled="editingFunc.isLocked"
                  compact
                  label="Description"
                  type="textarea"
                  @blur="updateFunc"
                />
              </Stack>
            </TreeNode>

            <ActionDetails
              v-if="editingFunc.kind === FuncKind.Action"
              ref="detachRef"
              :funcId="editingFunc.funcId"
              :schemaVariantId="$props.schemaVariantId"
            />
            <AuthenticationDetails
              v-if="editingFunc.kind === FuncKind.Authentication"
              ref="detachRef"
              :funcId="editingFunc.funcId"
              :schemaVariantId="$props.schemaVariantId"
            />
            <CodeGenerationDetails
              v-if="editingFunc.kind === FuncKind.CodeGeneration"
              ref="detachRef"
              :funcId="editingFunc.funcId"
              :schemaVariantId="$props.schemaVariantId"
            />
            <QualificationDetails
              v-if="editingFunc.kind === FuncKind.Qualification"
              ref="detachRef"
              :funcId="editingFunc.funcId"
              :schemaVariantId="$props.schemaVariantId"
            />

            <TreeNode
              v-if="editingFunc?.kind === FuncKind.Attribute"
              alwaysShowArrow
              childrenContainerClasses="border-b border-neutral-200 dark:border-neutral-600"
              defaultOpen
              enableGroupToggle
              indentationSize="none"
              label="Arguments"
              labelClasses="border-b border-neutral-200 dark:border-neutral-600"
              leftBorderSize="none"
            >
              <FuncArguments
                :disabled="editingFunc.isLocked"
                :funcId="editingFunc.funcId"
              />
            </TreeNode>

            <TreeNode
              v-if="
                editingFunc?.kind === FuncKind.Attribute &&
                $props.schemaVariantId
              "
              alwaysShowArrow
              childrenContainerClasses="border-b border-neutral-200 dark:border-neutral-600"
              defaultOpen
              enableGroupToggle
              indentationSize="none"
              label="Bindings"
              labelClasses="border-b border-neutral-200 dark:border-neutral-600"
              leftBorderSize="none"
            >
              <AttributeBindings
                ref="detachRef"
                :funcId="editingFunc.funcId"
                :schemaVariantId="$props.schemaVariantId"
              />
            </TreeNode>
          </div>
        </TabGroupItem>

        <TabGroupItem
          v-if="editingFunc?.kind === FuncKind.Attribute"
          label="Bindings"
          slug="bindings"
        >
          <AttributeBindings
            ref="detachRef"
            :funcId="editingFunc.funcId"
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
import {
  ErrorMessage,
  Stack,
  TabGroup,
  TabGroupItem,
  TreeNode,
  VormInput,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { FuncKind, FuncId, FuncBindingKind } from "@/api/sdf/dal/func";
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
import EditingPill from "../EditingPill.vue";

const props = defineProps<{
  funcId: FuncId;
  schemaVariantId: string;
  allowTestPanel?: boolean;
}>();

const funcDetailsTabGroupRef = ref();

const funcStore = useFuncStore();
const assetStore = useAssetStore();

const schemaVariant = computed(() => {
  return assetStore.variantFromListById[props.schemaVariantId];
});

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

watch(
  () => funcStore.selectedFuncSummary,
  () => {
    resetEditingFunc();
  },
);

const editingFunc = ref(_.cloneDeep(funcStore.selectedFuncSummary));

function resetEditingFunc() {
  editingFunc.value = _.cloneDeep(funcStore.selectedFuncSummary);
}

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

const updateFunc = async () => {
  if (editingFunc.value) {
    await funcStore.UPDATE_FUNC(editingFunc.value);
    resetEditingFunc();
  }
};

const unlocking = ref(false);
const unlock = async () => {
  if (editingFunc.value?.funcId === undefined) return;

  unlocking.value = true;
  const resp = await funcStore.CREATE_UNLOCKED_COPY(
    editingFunc.value.funcId,
    assetStore.selectedVariantId,
  );
  if (resp.result.success) {
    try {
      let unlockedAssetId =
        assetStore.unlockedVariantIdForId[assetStore.selectedVariantId ?? ""];
      if (!unlockedAssetId) {
        await assetStore.LOAD_SCHEMA_VARIANT_LIST();
      }
      unlockedAssetId =
        assetStore.unlockedVariantIdForId[assetStore.selectedVariantId ?? ""];

      if (!unlockedAssetId) {
        unlocking.value = false;
        throw Error("Unlocked asset without unlocking variant?");
      }

      assetStore.setSchemaVariantSelection(unlockedAssetId);

      await assetStore.setFuncSelection(resp.result.data.summary.funcId);
    } catch (err) {
      unlocking.value = false;
      throw err;
    }
  }

  unlocking.value = false;
};

const isConnectedToOtherSchemas = computed<boolean>(() => {
  if (!editingFunc.value) return false;

  if (editingFunc.value.bindings.length === 1) return false;

  // TODO this is wrong for attribute funcs, since they can have multiple bindings on the same variant
  return true;
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
    if (funcStore.selectedFuncId) assetStore.setFuncSelection(undefined);
  }
};

const isDeleting = ref(false);
const deleteFunc = async () => {
  if (!funcId.value) return;
  await funcStore.DELETE_UNLOCKED_FUNC(funcId.value);
  assetStore.setFuncSelection(undefined);
};

/* dont think we need this anymore
const hasAssociations = computed(() => {
  if (editingFunc.value?.bindings.length === 0) return true;
  return false;
});
*/

// The parent component can allow the test panel to be enabled, but we need to dynamically enable
// it based on the func kind.
const enableTestPanel = computed(() => {
  return (
    props.allowTestPanel &&
    editingFunc.value &&
    editingFunc.value.bindings
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
