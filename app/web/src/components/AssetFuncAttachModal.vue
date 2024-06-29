<template>
  <Modal
    ref="modalRef"
    :title="title"
    :size="attachExisting && selectedExistingFuncId ? '4xl' : 'md'"
    @close="onClose"
  >
    <div class="flex flex-row max-h-[75vh]">
      <div
        :class="
          clsx(
            'flex flex-col gap-y-4 min-w-[250px]',
            attachExisting && selectedExistingFuncId ? 'mr-sm' : 'flex-grow',
          )
        "
      >
        <VormInput
          v-model="funcKind"
          label="Kind"
          type="dropdown"
          :options="funcKindOptions"
        />
        <VormInput
          v-if="!attachExisting"
          v-model="name"
          label="Name"
          required
          placeholder="The name of the function"
        />
        <VormInput
          v-if="attachExisting"
          v-model="selectedExistingFuncId"
          label="Existing function"
          type="dropdown"
          required
          :options="existingFuncOptions"
        />
        <template v-if="funcKind === FuncKind.Action">
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
        </template>
        <VormInput
          v-if="funcKind === FuncKind.Attribute"
          v-model="attributeOutputLocation"
          label="Output Location"
          type="dropdown"
          required
          :options="attributeOutputLocationOptions"
        />
        <ErrorMessage
          v-if="createFuncReqStatus.isError && createFuncStarted"
          :requestStatus="createFuncReqStatus"
        />
        <template v-if="attachExisting && funcKind === FuncKind.Attribute">
          <h1 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
            Expected Function Arguments:
          </h1>
          <h2 class="text-sm">
            Below is the source of the data for each function argument listed.
          </h2>
          <ul>
            <li
              v-for="binding in editableBindings"
              :key="binding.funcArgumentId"
            >
              <h1
                class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50"
              >
                {{ funcArgumentName(binding.funcArgumentId) ?? "none" }}
              </h1>
              <SelectMenu
                v-model="binding.binding"
                :options="inputSourceOptions"
              />
            </li>
          </ul>
        </template>
        <VButton
          class="w-full"
          :loading="showLoading"
          :disabled="!attachEnabled"
          :loadingText="`Attaching ${existingOrNew} function...`"
          :label="`Attach ${existingOrNew} function`"
          tone="action"
          icon="plus"
          size="sm"
          @click="onAttach"
        />
      </div>
      <div
        v-if="attachExisting && selectedExistingFuncId"
        class="overflow-y-scroll"
      >
        <div v-if="loadFuncDetailsReq?.value.isPending">
          <RequestStatusMessage :requestStatus="loadFuncDetailsReq.value" />
        </div>
        <CodeEditor
          v-if="loadFuncDetailsReq && !loadFuncDetailsReq?.value.isPending"
          v-model="selectedFuncCode"
          disabled
          typescript="yes"
          noLint
          noVim
        />
      </div>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import {
  ErrorMessage,
  Modal,
  RequestStatusMessage,
  useModal,
  VButton,
  VormInput,
} from "@si/vue-lib/design-system";
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
} from "@/api/sdf/dal/func";
import {
  outputSocketsAndPropsFor,
  inputSocketsAndPropsFor,
} from "@/api/sdf/dal/schema";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { nilId } from "@/utils/nilId";
import CodeEditor from "./CodeEditor.vue";

const props = defineProps<{
  assetId?: string;
}>();

const funcStore = useFuncStore();
const assetStore = useAssetStore();

const createFuncStarted = ref(false);

const createFuncReqStatus = funcStore.getRequestStatus("CREATE_FUNC");
const loadAssetsReqStatus = assetStore.getRequestStatus(
  "LOAD_SCHEMA_VARIANT",
  props.assetId,
);

const schemaVariantId = computed(() =>
  props.assetId
    ? assetStore.variantsById[props.assetId]?.schemaVariantId
    : undefined,
);

const showLoading = computed(
  () =>
    createFuncReqStatus.value.isPending || loadAssetsReqStatus.value.isPending,
);

const funcKindOptions = Object.keys(CUSTOMIZABLE_FUNC_TYPES).map((kind) => ({
  label: CUSTOMIZABLE_FUNC_TYPES[kind as CustomizableFuncKind]?.singularLabel,
  value: kind as string,
}));

const attachExisting = ref(false);

const name = ref("");
const funcKind = ref(FuncKind.Action);

const selectedExistingFuncId = ref<FuncId | undefined>();
const selectedExistingFuncSummary = computed(() => {
  if (selectedExistingFuncId.value)
    return funcStore.funcsById[selectedExistingFuncId.value];

  return undefined;
});
const selectedFuncCode = ref<string>("");
const loadFuncDetailsReq = computed(() =>
  selectedExistingFuncId.value
    ? funcStore.getRequestStatus("FETCH_CODE", selectedExistingFuncId.value)
    : undefined,
);

watch(
  selectedExistingFuncId,
  async (funcId) => {
    if (funcId) {
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
    .filter((func) => func.kind === funcKind.value)
    .map((func) => ({
      label: func.name,
      value: func.funcId,
    })),
);

const attributeOutputLocation = ref<string | undefined>();
const attributeOutputLocationOptions = ref<Option[]>([]);

const attrToValidate = ref<string | undefined>();

const assetName = computed(
  () =>
    assetStore.selectedSchemaVariant?.displayName ??
    assetStore.selectedSchemaVariant?.schemaName ??
    " none",
);

const existingOrNew = computed(() =>
  attachExisting.value ? "existing" : "new",
);

const title = computed(() =>
  assetName.value
    ? `Attach ${existingOrNew.value} function to "${assetName.value}"`
    : `Attach ${existingOrNew.value} function`,
);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const attachEnabled = computed(() => {
  const nameIsSet =
    attachExisting.value || !!(name.value && name.value.length > 0);
  const hasOutput =
    funcKind.value !== FuncKind.Attribute || !!attributeOutputLocation.value;
  const existingSelected =
    !attachExisting.value || !!selectedExistingFuncId.value;
  const argsConfigured =
    !attachExisting.value ||
    funcKind.value !== FuncKind.Attribute ||
    editableBindings.value.every((b) => b.binding.value !== nilId());

  return nameIsSet && hasOutput && existingSelected && argsConfigured;
});

const open = async (
  existing?: boolean,
  variant?: FuncKind,
  funcId?: FuncId,
) => {
  attachExisting.value = existing ?? false;

  attributeOutputLocation.value = "";

  name.value = "";
  funcKind.value = variant ?? FuncKind.Action;
  isCreate.value = false;
  isDelete.value = false;
  isRefresh.value = false;
  selectedFuncCode.value = "";
  selectedExistingFuncId.value = funcId;
  attrToValidate.value = undefined;

  attributeOutputLocationOptions.value = [];
  if (props.assetId) {
    const schemaVariant = assetStore.variantsById[props.assetId];
    if (schemaVariant) {
      const { socketOptions, propOptions } =
        outputSocketsAndPropsFor(schemaVariant);

      attributeOutputLocationOptions.value = [...socketOptions, ...propOptions];
    }
  }

  openModal();
};

// NOT SURE I NEED THIS
const reloadAssetAndRoute = async (assetId: string) => {
  await assetStore.LOAD_SCHEMA_VARIANT(assetId);
  close();
};

interface EditingBinding {
  funcArgumentId: string;
  binding: Option;
}

const editableBindings = ref<EditingBinding[]>([]);
const inputSourceOptions = computed<Option[]>(() => {
  let socketOptions: Option[] = [];
  let propOptions: Option[] = [];
  if (schemaVariantId.value) {
    const variant = assetStore.variantsById[schemaVariantId.value];
    if (variant) {
      ({ socketOptions, propOptions } = inputSocketsAndPropsFor(variant));
    }
  }

  return socketOptions.concat(propOptions);
});
const funcArgumentName = (
  funcArgumentId: FuncArgumentId,
): string | undefined => {
  return selectedExistingFuncSummary.value?.arguments
    .filter((a) => a.id === funcArgumentId)
    .pop()?.name;
};
const noneSource = { label: "select source", value: nilId() };

const commonBindingConstruction = () => {
  const binding = {
    funcId: selectedExistingFuncId.value,
    schemaVariantId: schemaVariantId.value,
  } as unknown;
  switch (funcKind.value) {
    case FuncKind.Authentication:
      // eslint-disable-next-line no-case-declarations
      const auth = binding as Authentication;
      auth.bindingKind = FuncBindingKind.Authentication;
      return auth;
    case FuncKind.Action:
      // eslint-disable-next-line no-case-declarations
      const action = binding as Action;
      action.bindingKind = FuncBindingKind.Action;
      if (isCreate.value) action.kind = ActionKind.Create;
      if (isDelete.value) action.kind = ActionKind.Destroy;
      if (isRefresh.value) action.kind = ActionKind.Refresh;
      if (!isRefresh.value && !isDelete.value && !isCreate.value)
        action.kind = ActionKind.Manual;
      return action;
    case FuncKind.CodeGeneration:
      // eslint-disable-next-line no-case-declarations
      const bind = binding as CodeGeneration;
      bind.inputs = [];
      bind.bindingKind = FuncBindingKind.CodeGeneration;
      return bind;
    case FuncKind.Qualification:
      // eslint-disable-next-line no-case-declarations
      const qual = binding as Qualification;
      qual.inputs = [];
      qual.bindingKind = FuncBindingKind.Qualification;
      return qual;
    default:
      return null;
  }
};

const attachExistingFunc = async () => {
  const bindings: FuncBinding[] = [];

  const binding = commonBindingConstruction();
  if (binding) bindings.push(binding);

  if (funcKind.value === FuncKind.Attribute) {
    const attr = {
      funcId: selectedExistingFuncId.value,
      schemaVariantId: schemaVariantId.value,
      bindingKind: FuncBindingKind.Attribute,
    } as Attribute;
    const argBindings: AttributeArgumentBinding[] = [];
    editableBindings.value.forEach(async (b) => {
      const arg = {
        funcArgumentId: b.funcArgumentId,
      } as AttributeArgumentBinding;
      const bString = b.binding.value as string;
      if (bString.startsWith("s_"))
        arg.inputSocketId = bString.replace("s_", "");
      else if (bString.startsWith("p_")) arg.propId = bString.replace("p_", "");

      argBindings.push(arg);
    });
    attr.argumentBindings = argBindings;
    if (attributeOutputLocation.value?.startsWith("s_"))
      attr.outputSocketId = attributeOutputLocation.value.replace("s_", "");
    else if (attributeOutputLocation.value?.startsWith("p_"))
      attr.propId = attributeOutputLocation.value.replace("p_", "");

    bindings.push(attr);
  }
  if (bindings.length > 0 && selectedExistingFuncId.value) {
    const response = await funcStore.CREATE_BINDING(
      selectedExistingFuncId.value,
      bindings,
    );
    if (response.result.success && props.assetId) {
      await reloadAssetAndRoute(props.assetId);
    }
  }
};

const attachNewFunc = async () => {
  if (schemaVariantId.value) {
    createFuncStarted.value = true;
    let binding = commonBindingConstruction() as FuncBinding;
    if (!binding) {
      let outputSocketId;
      let propId;
      if (attributeOutputLocation.value?.startsWith("s_"))
        outputSocketId = attributeOutputLocation.value.replace("s_", "");
      else if (attributeOutputLocation.value?.startsWith("p_"))
        propId = attributeOutputLocation.value.replace("p_", "");

      binding = {
        funcId: selectedExistingFuncId.value,
        schemaVariantId: schemaVariantId.value,
        bindingKind: FuncBindingKind.Attribute,
        argumentBindings: [],
        propId,
        outputSocketId,
        componentId: null,
        attributePrototypeId: null,
      } as Attribute;
    }
    const result = await funcStore.CREATE_FUNC({
      name: name.value,
      displayName: name.value,
      description: "",
      binding,
      kind: funcKind.value,
    });
    if (result.result.success) {
      funcStore.selectedFuncId = result.result.data.summary.funcId;
      assetStore.addFuncSelection(result.result.data.summary.funcId);
      if (props.assetId) await reloadAssetAndRoute(props.assetId);
    }
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

const setCreate = () => {
  if (!isCreate.value) return;
  isDelete.value = false;
  isRefresh.value = false;
};

const setRefresh = () => {
  if (!isRefresh.value) return;
  isCreate.value = false;
  isDelete.value = false;
};

const setDelete = () => {
  if (!isDelete.value) return;
  isCreate.value = false;
  isRefresh.value = false;
};
</script>
