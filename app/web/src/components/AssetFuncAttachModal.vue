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
        <template v-if="funcKind === FuncKind.Action && !attachExisting">
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
import { ActionKind } from "@/store/actions.store";
import SiCheckBox from "@/components/SiCheckBox.vue";
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
  FuncKind,
  FuncId,
  FuncArgumentId,
} from "@/api/sdf/dal/func";
import SelectMenu, { Option } from "@/components/SelectMenu.vue";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import {
  CreateFuncOptions,
  CreateFuncOutputLocation,
  FuncAssociations,
} from "@/store/func/types";
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
const selectedFuncCode = ref<string>("");
const loadFuncDetailsReq = computed(() =>
  selectedExistingFuncId.value
    ? funcStore.getRequestStatus("FETCH_FUNC", selectedExistingFuncId.value)
    : undefined,
);

watch(
  selectedExistingFuncId,
  async (funcId) => {
    if (funcId) {
      if (
        !funcStore.funcDetailsById[funcId] ||
        !funcStore.funcArgumentsByFuncId[funcId]
      ) {
        await funcStore.FETCH_FUNC_ARGUMENT_LIST(funcId);
      } else {
        selectedFuncCode.value = funcStore.funcDetailsById[funcId]?.code ?? "";
      }

      editableBindings.value = [];
      if (func.value?.associations?.type === "attribute") {
        editableBindings.value =
          funcStore.funcArgumentsByFuncId[func.value.id]?.map((a) => ({
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
      value: func.id,
    })),
);

// VormInput does not support object types for option values, so we do transform
// the value into JSON and parse it back to use it
const attributeOutputLocation = ref<string | undefined>();
const attributeOutputLocationParsed = computed<
  CreateFuncOutputLocation | undefined
>(() => {
  const parsed = attributeOutputLocation.value
    ? JSON.parse(attributeOutputLocation.value)
    : undefined;

  if (parsed) {
    if ("propId" in parsed && parsed.propId) {
      return {
        type: "prop",
        propId: parsed.propId,
      };
    } else if ("outputSocketId" in parsed && parsed.outputSocketId) {
      return {
        type: "outputSocket",
        outputSocketId: parsed.outputSocketId,
      };
    }
  }

  return undefined;
});
const attributeOutputLocationOptions = ref<{ label: string; value: string }[]>(
  [],
);

const attrToValidate = ref<string | undefined>();
const validationOptions = ref<{ label: string; value: string }[]>([]);

const assetName = computed(
  () => assetStore.selectedSchemaVariant?.schemaName ?? " none",
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
    funcKind.value !== FuncKind.Attribute ||
    !!attributeOutputLocationParsed.value;
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

  await funcStore.FETCH_INPUT_SOURCE_LIST(schemaVariantId.value);
  attributeOutputLocationOptions.value = schemaVariantId.value
    ? funcStore
        .outputLocationOptionsForSchemaVariant(schemaVariantId.value)
        .map(({ label, value }) => ({
          label,
          value: JSON.stringify(value),
        }))
    : [];

  validationOptions.value = schemaVariantId.value
    ? funcStore.propsAsOptionsForSchemaVariant(schemaVariantId.value)
    : [];

  openModal();
};

const newFuncOptions = (
  funcKind: FuncKind,
  schemaVariantId: string,
): CreateFuncOptions => {
  const baseOptions = {
    schemaVariantId,
  };

  let kind = ActionKind.Manual;
  switch (funcKind) {
    case FuncKind.Authentication:
      return {
        type: "authenticationOptions",
        ...baseOptions,
      };
    case FuncKind.Action:
      if (isCreate.value) kind = ActionKind.Create;
      if (isDelete.value) kind = ActionKind.Destroy;
      if (isRefresh.value) kind = ActionKind.Refresh;

      return {
        type: "actionOptions",
        actionKind: kind,
        ...baseOptions,
      };
    case FuncKind.Attribute:
      if (attributeOutputLocationParsed.value) {
        return {
          type: "attributeOptions",
          outputLocation: attributeOutputLocationParsed.value,
        };
      }
      throw new Error(
        `attributeOutputLocationParsed not defined for Attribute`,
      );

    case FuncKind.CodeGeneration:
      return {
        type: "codeGenerationOptions",
        ...baseOptions,
      };
    case FuncKind.Qualification:
      return {
        type: "qualificationOptions",
        ...baseOptions,
      };
    default:
      throw new Error(`newFuncOptions not defined for ${funcKind}`);
  }
};

const attachToAttributeFunction = async (
  outputLocation: CreateFuncOutputLocation,
) => {
  if (!selectedExistingFuncId.value || !schemaVariantId.value) return;
  const prototypes = editableBindings.value.map((b) => ({
    id: nilId(),
    funcArgumentId: b.funcArgumentId,
    inputSocketId: b.binding.label.includes("Input Socket")
      ? (b.binding.value as string)
      : undefined,
    propId: b.binding.label.includes("Attribute")
      ? (b.binding.value as string)
      : undefined,
  }));
  await funcStore.CREATE_ATTRIBUTE_PROTOTYPE(
    selectedExistingFuncId.value,
    schemaVariantId.value,
    prototypes,
    nilId(),
    "propId" in outputLocation ? outputLocation.propId : undefined,
    "outputSocketId" in outputLocation
      ? outputLocation.outputSocketId
      : undefined,
  );
};

const reloadAssetAndRoute = async (assetId: string) => {
  await assetStore.LOAD_SCHEMA_VARIANT(assetId);
  close();
};

const func = computed(
  () => funcStore.funcDetailsById[selectedExistingFuncId.value ?? -1],
);

interface EditingBinding {
  funcArgumentId: string;
  binding: Option;
}

const editableBindings = ref<EditingBinding[]>([]);
const inputSourceOptions = computed<Option[]>(() => {
  const selectedVariantId = schemaVariantId.value ?? -1;
  const socketOptions =
    funcStore.inputSourceSockets[selectedVariantId]?.map((socket) => ({
      label: `Input Socket: ${socket.name}`,
      value: socket.inputSocketId,
    })) ?? [];

  const propOptions =
    funcStore.inputSourceProps[selectedVariantId]?.map((prop) => ({
      label: `Attribute: ${prop.path}`,
      value: prop.propId,
    })) ?? [];

  return socketOptions.concat(propOptions);
});
const funcArgumentName = (
  funcArgumentId: FuncArgumentId,
): string | undefined => {
  return funcStore.funcArgumentsById[funcArgumentId]?.name;
};
const noneSource = { label: "select source", value: nilId() };

const attachExistingFunc = async () => {
  if (schemaVariantId.value && selectedExistingFuncId.value) {
    const func = funcStore.funcDetailsById[selectedExistingFuncId.value];
    if (func) {
      let updatedAssociations: FuncAssociations | undefined;
      const associations = func?.associations;
      if (!associations) {
        return;
      }

      switch (associations.type) {
        case "authentication":
        case "action":
        case "codeGeneration":
        case "qualification":
          updatedAssociations = _.cloneDeep(associations);
          updatedAssociations.schemaVariantIds.push(schemaVariantId.value);
          break;
        case "attribute":
          if (attributeOutputLocationParsed.value) {
            attachToAttributeFunction(attributeOutputLocationParsed.value);
            if (props.assetId) await reloadAssetAndRoute(props.assetId);
          }
          break;
        default:
          throw new Error(
            `type "${
              (associations as FuncAssociations)?.type
            }" is not supported by attachExistingFunc`,
          );
      }
      if (updatedAssociations) {
        func.associations = updatedAssociations;
        const response = await funcStore.UPDATE_FUNC(func);
        if (response.result.success && props.assetId) {
          await reloadAssetAndRoute(props.assetId);
        }
      }
    }
  }
};

const attachNewFunc = async () => {
  if (schemaVariantId.value) {
    const options = newFuncOptions(funcKind.value, schemaVariantId.value);
    createFuncStarted.value = true;
    const result = await funcStore.CREATE_FUNC({
      kind: funcKind.value,
      name: name.value,
      options,
    });
    if (result.result.success) {
      funcStore.selectedFuncId = result.result.data.id;
      assetStore.addFuncSelection(result.result.data.id);
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
