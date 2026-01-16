<template>
  <Modal
    ref="modalRef"
    :size="attachExisting && selectedExistingFunc.value !== nilId() ? '4xl' : 'md'"
    :title="title"
    @close="onClose"
  >
    <div class="flex flex-row max-h-[75vh]">
      <div
        :class="
          clsx(
            'flex flex-col gap-y-4 min-w-[250px]',
            attachExisting && selectedExistingFunc.value !== nilId() ? 'mr-sm' : 'flex-grow',
          )
        "
      >
        <SelectMenu v-model="funcKind" :options="funcKindOptions" label="Kind" type="dropdown" />
        <VormInput v-if="!attachExisting" v-model="name" label="Name" placeholder="The name of the function" required />
        <SelectMenu
          v-if="attachExisting"
          v-model="selectedExistingFunc"
          :options="existingFuncOptions"
          label="Existing function"
          required
          type="dropdown"
          canFilter
        />
        <template v-if="funcKind.value === FuncKind.Action">
          <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
            <SiCheckBox
              id="create"
              v-model="isCreate"
              title="This action creates a resource"
              @update:model-value="setCreate"
            />
          </div>
          <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
            <SiCheckBox
              id="refresh"
              v-model="isRefresh"
              title="This action refreshes a resource"
              @update:model-value="setRefresh"
            />
          </div>
          <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
            <SiCheckBox
              id="delete"
              v-model="isDelete"
              title="This action deletes a resource"
              @update:model-value="setDelete"
            />
          </div>
          <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
            <SiCheckBox
              id="update"
              v-model="isUpdate"
              title="This action updates a resource"
              @update:model-value="setUpdate"
            />
          </div>
        </template>
        <SelectMenu
          v-if="funcKind.value === FuncKind.Attribute"
          v-model="attributeOutputLocation"
          :options="attributeOutputLocationOptions"
          label="Output Location"
          required
          type="dropdown"
          canFilter
        />
        <ErrorMessage v-if="createFuncReqStatus.isError && createFuncStarted" :requestStatus="createFuncReqStatus" />
        <template v-if="attachExisting && funcKind.value === FuncKind.Attribute">
          <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">Expected Function Arguments:</h1>
          <h2 class="text-sm">Below is the source of the data for each function argument listed.</h2>
          <ul>
            <li v-for="binding in editableBindings" :key="binding.funcArgumentId">
              <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
                {{ funcArgumentName(binding.funcArgumentId) ?? "none" }}
              </h1>
              <SelectMenu v-model="binding.binding" :options="inputSourceOptions" canFilter />
            </li>
          </ul>
        </template>
        <VButton
          :disabled="!attachEnabled"
          :label="`Attach ${existingOrNew} function`"
          :loading="showLoading"
          :loadingText="`Attaching ${existingOrNew} function...`"
          class="w-full"
          icon="plus"
          size="sm"
          tone="action"
          @click="onAttach"
        />
      </div>
      <div v-if="attachExisting && selectedExistingFunc.value !== nilId()" class="overflow-y-scroll">
        <div v-if="loadFuncDetailsReq?.value.isPending">
          <RequestStatusMessage :requestStatus="loadFuncDetailsReq.value" />
        </div>
        <CodeEditor
          v-if="loadFuncDetailsReq && loadFuncDetailsReq?.value.isSuccess && selectedFuncCode"
          :id="codeEditorId"
          v-model="selectedFuncCode"
          :recordId="selectedExistingFunc.value as string"
          disabled
          noLint
          noVim
          typescript="yes"
        />
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import { ErrorMessage, Modal, RequestStatusMessage, useModal, VButton, VormInput } from "@si/vue-lib/design-system";
import clsx from "clsx";
import * as _ from "lodash-es";
import { ActionKind } from "@/api/sdf/dal/action";
import SiCheckBox from "@/components/SiCheckBox.vue";
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
  FuncKind,
  FuncId,
  FuncArgumentId,
  FuncBindingKind,
  FuncBinding,
  Authentication,
  CodeGeneration,
  Action,
  Attribute,
  Qualification,
  AttributeArgumentBinding,
  Management,
} from "@/api/sdf/dal/func";
import { outputSocketsAndPropsFor, inputSocketsAndPropsFor, SchemaVariantId } from "@/api/sdf/dal/schema";
import SelectMenu, { Option, GroupedOptions } from "@/components/SelectMenu.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { nilId } from "@/utils/nilId";
import CodeEditor from "./CodeEditor.vue";

const props = defineProps<{
  schemaVariantId?: SchemaVariantId;
}>();

const funcStore = useFuncStore();
const assetStore = useAssetStore();

const createFuncStarted = ref(false);

const createFuncReqStatus = funcStore.getRequestStatus("CREATE_FUNC");

const schemaVariantId = computed(() =>
  props.schemaVariantId ? assetStore.variantFromListById[props.schemaVariantId]?.schemaVariantId : undefined,
);

const showLoading = computed(() => createFuncReqStatus.value.isPending);

const funcKindOptions = Object.keys(CUSTOMIZABLE_FUNC_TYPES).map((kind) => ({
  label: CUSTOMIZABLE_FUNC_TYPES[kind as CustomizableFuncKind]?.singularLabel,
  value: kind as FuncKind,
}));

const attachExisting = ref(false);

const name = ref("");
const funcKind = ref({ label: "Actions", value: FuncKind.Action });

const noneFunction = {
  label: "select function",
  value: nilId(),
};
const selectedExistingFunc = ref<Option>(noneFunction);
const codeEditorId = computed(() => `func-${selectedExistingFunc.value.value}`);
const selectedExistingFuncSummary = computed(() => funcStore.funcsById[selectedExistingFunc.value.value as string]);
const selectedFuncCode = ref<string>("");
const loadFuncDetailsReq = computed(() => {
  const id = selectedExistingFunc.value.value as string;
  if (id !== nilId()) return funcStore.getRequestStatus("FETCH_CODE", id);
  return undefined;
});

watch(
  selectedExistingFunc,
  async ({ value }) => {
    const funcId = value as string;
    if (funcId !== nilId()) {
      await funcStore.FETCH_CODE(funcId);
      selectedFuncCode.value = funcStore.funcCodeById[funcId]?.code || "";

      editableBindings.value = [];
      if (selectedExistingFuncSummary.value?.kind === FuncKind.Attribute) {
        editableBindings.value =
          selectedExistingFuncSummary.value.arguments?.map((a) => ({
            funcArgumentId: a.id,
            binding: noneSource,
          })) ?? [];
      }
    }
  },
  { immediate: true },
);

const existingFuncOptions = computed(() =>
  Object.values(funcStore.funcsById)
    .filter((func) => func.kind === funcKind.value.value)
    .map((func) => ({
      label: func.name,
      value: func.funcId,
    })),
);

const noneOutput = {
  label: "select output location",
  value: nilId(),
};
const attributeOutputLocation = ref<Option>(noneOutput);
const attributeOutputLocationOptions = ref<GroupedOptions>({});

const attrToValidate = ref<string | undefined>();

const assetName = computed(
  () => assetStore.selectedSchemaVariant?.displayName ?? assetStore.selectedSchemaVariant?.schemaName ?? " none",
);

const existingOrNew = computed(() => (attachExisting.value ? "existing" : "new"));

const title = computed(() =>
  assetName.value
    ? `Attach ${existingOrNew.value} function to "${assetName.value}"`
    : `Attach ${existingOrNew.value} function`,
);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const attachEnabled = computed(() => {
  const nameIsSet = attachExisting.value || !!(name.value && name.value.length > 0);
  const hasOutput = funcKind.value.value !== FuncKind.Attribute || !!attributeOutputLocation.value;
  const existingSelected = !attachExisting.value || selectedExistingFunc.value.value !== nilId();
  const argsConfigured =
    !attachExisting.value ||
    funcKind.value.value !== FuncKind.Attribute ||
    editableBindings.value.every((b) => b.binding.value !== nilId());

  return nameIsSet && hasOutput && existingSelected && argsConfigured;
});

const open = async (existing?: boolean, variant?: FuncKind, funcId?: FuncId) => {
  attachExisting.value = existing ?? false;

  attributeOutputLocation.value = noneOutput;

  name.value = "";
  funcKind.value = funcKindOptions.find((o) => o.value === variant) ?? {
    label: "Actions",
    value: FuncKind.Action,
  };
  isCreate.value = false;
  isDelete.value = false;
  isRefresh.value = false;
  isUpdate.value = false;
  selectedFuncCode.value = "";
  selectedExistingFunc.value = existingFuncOptions.value.find((o) => o.value === funcId) || noneFunction;
  attrToValidate.value = undefined;

  attributeOutputLocationOptions.value = {};
  if (props.schemaVariantId) {
    const schemaVariant = assetStore.variantFromListById[props.schemaVariantId];
    if (schemaVariant) {
      attributeOutputLocationOptions.value = outputSocketsAndPropsFor(schemaVariant);
    }
  }

  openModal();
};

interface EditingBinding {
  funcArgumentId: string;
  binding: Option;
}

const editableBindings = ref<EditingBinding[]>([]);
const inputSourceOptions = computed<GroupedOptions>(() => {
  if (schemaVariantId.value) {
    const variant = assetStore.variantFromListById[schemaVariantId.value];
    if (variant) {
      return inputSocketsAndPropsFor(variant);
    }
  }

  return {};
});
const funcArgumentName = (funcArgumentId: FuncArgumentId): string | undefined => {
  return selectedExistingFuncSummary.value?.arguments.filter((a) => a.id === funcArgumentId).pop()?.name;
};
const noneSource = { label: "select source", value: nilId() };

const commonBindingConstruction = () => {
  const binding = {
    funcId: selectedExistingFunc.value.value as string,
    schemaVariantId: schemaVariantId.value,
  } as unknown;
  switch (funcKind.value.value) {
    case FuncKind.Authentication:
      const auth = binding as Authentication;
      auth.bindingKind = FuncBindingKind.Authentication;
      return auth;
    case FuncKind.Action:
      const action = binding as Action;
      action.bindingKind = FuncBindingKind.Action;
      if (isCreate.value) action.kind = ActionKind.Create;
      if (isUpdate.value) action.kind = ActionKind.Update;
      if (isDelete.value) action.kind = ActionKind.Destroy;
      if (isRefresh.value) action.kind = ActionKind.Refresh;
      if (!isRefresh.value && !isDelete.value && !isCreate.value && !isUpdate.value) action.kind = ActionKind.Manual;
      return action;
    case FuncKind.CodeGeneration:
      const bind = binding as CodeGeneration;
      bind.inputs = [];
      bind.bindingKind = FuncBindingKind.CodeGeneration;
      return bind;
    case FuncKind.Qualification:
      const qual = binding as Qualification;
      qual.inputs = [];
      qual.bindingKind = FuncBindingKind.Qualification;
      return qual;
    case FuncKind.Management:
      const mgmt = binding as Management;
      mgmt.bindingKind = FuncBindingKind.Management;
      return mgmt;
    default:
      return null;
  }
};

const attachExistingFunc = async () => {
  const bindings: FuncBinding[] = [];

  const binding = commonBindingConstruction();
  if (binding) bindings.push(binding);

  if (funcKind.value.value === FuncKind.Attribute) {
    const attr = {
      funcId: selectedExistingFunc.value.value as string,
      schemaVariantId: schemaVariantId.value,
      bindingKind: FuncBindingKind.Attribute,
    } as Attribute;
    const argBindings: AttributeArgumentBinding[] = [];
    editableBindings.value.forEach(async (b) => {
      const arg = {
        funcArgumentId: b.funcArgumentId,
      } as AttributeArgumentBinding;
      const bString = b.binding.value as string;
      if (bString.startsWith("s_")) arg.inputSocketId = bString.replace("s_", "");
      else if (bString.startsWith("p_")) arg.propId = bString.replace("p_", "");

      argBindings.push(arg);
    });
    attr.argumentBindings = argBindings;
    const loc = attributeOutputLocation.value.value as string;
    if (loc.startsWith("s_")) attr.outputSocketId = loc.replace("s_", "");
    else if (loc.startsWith("p_")) attr.propId = loc.replace("p_", "");

    bindings.push(attr);
  }
  if (bindings.length > 0 && selectedExistingFunc.value.value !== nilId()) {
    const resp = await funcStore.CREATE_BINDING(selectedExistingFunc.value.value as string, bindings);
    if (resp.result.success) close();
  }
};

const attachNewFunc = async () => {
  if (schemaVariantId.value) {
    createFuncStarted.value = true;
    let binding = commonBindingConstruction() as FuncBinding;
    if (!binding) {
      let outputSocketId;
      let propId;
      const loc = attributeOutputLocation.value.value as string;
      if (loc.startsWith("s_")) outputSocketId = loc.replace("s_", "");
      else if (loc.startsWith("p_")) propId = loc.replace("p_", "");

      binding = {
        funcId: selectedExistingFunc.value.value as string,
        schemaVariantId: schemaVariantId.value,
        bindingKind: FuncBindingKind.Attribute,
        argumentBindings: [],
        propId,
        outputSocketId,
        componentId: null,
        attributePrototypeId: null,
      } as Attribute;
    }
    const resp = await funcStore.CREATE_FUNC({
      name: name.value,
      displayName: name.value,
      description: "",
      binding,
      kind: funcKind.value.value,
    });
    if (resp.result.success) close();
  }
};

const onAttach = async () => {
  if (attachExisting.value) {
    await attachExistingFunc();
  } else {
    await attachNewFunc();
  }
};

const onClose = () => {
  createFuncStarted.value = false;
};

defineExpose({ open, close });

const isCreate = ref(false);
const isDelete = ref(false);
const isRefresh = ref(false);
const isUpdate = ref(false);

const setCreate = () => {
  if (!isCreate.value) return;
  isDelete.value = false;
  isRefresh.value = false;
  isUpdate.value = false;
};

const setRefresh = () => {
  if (!isRefresh.value) return;
  isCreate.value = false;
  isDelete.value = false;
  isUpdate.value = false;
};

const setDelete = () => {
  if (!isDelete.value) return;
  isCreate.value = false;
  isRefresh.value = false;
  isUpdate.value = false;
};

const setUpdate = () => {
  if (!isUpdate.value) return;
  isCreate.value = false;
  isDelete.value = false;
  isRefresh.value = false;
};
</script>
