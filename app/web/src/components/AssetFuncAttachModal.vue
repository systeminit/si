<template>
  <Modal ref="modalRef" :title="title" :size="attachExisting ? '4xl' : 'md'">
    <div class="flex flex-row h-96">
      <div
        :class="
          clsx(
            'flex flex-col gap-y-4 min-w-[250px]',
            attachExisting && 'mr-3',
            !attachExisting && 'flex-grow',
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
          <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
            <SiCheckBox
              id="create"
              v-model="isCreate"
              title="This action creates a resource"
              @update:model-value="setCreate"
            />
          </h2>
          <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
            <SiCheckBox
              id="refresh"
              v-model="isRefresh"
              title="This action refreshes a resource"
              @update:model-value="setRefresh"
            />
          </h2>
          <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
            <SiCheckBox
              id="delete"
              v-model="isDelete"
              title="This action deletes a resource"
              @update:model-value="setDelete"
            />
          </h2>
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
          v-if="createFuncReqStatus.isError"
          :requestStatus="createFuncReqStatus"
        />
        <div class="mt-auto">
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
      </div>
      <div v-if="attachExisting" class="overflow-y-scroll">
        <div
          v-if="!selectedExistingFuncId"
          class="items-center justify-center text-neutral-400 dark:text-neutral-300 text-sm text-center"
        >
          Select an existing function to attach it to this asset
        </div>
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
import { useRouter } from "vue-router";
import clsx from "clsx";
import * as _ from "lodash-es";
import { ActionKind } from "@/store/actions.store";
import SiCheckBox from "@/components/SiCheckBox.vue";
import {
  CUSTOMIZABLE_FUNC_TYPES,
  CustomizableFuncKind,
  FuncKind,
} from "@/api/sdf/dal/func";
import { FuncId, useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import {
  AttributeAssociations,
  CreateFuncOptions,
  CreateFuncOutputLocation,
  FuncAssociations,
} from "@/store/func/types";
import { nilId } from "@/utils/nilId";
import CodeEditor from "./CodeEditor.vue";

const props = defineProps<{
  schemaVariantId?: string;
  assetId?: string;
}>();

const funcStore = useFuncStore();
const assetStore = useAssetStore();
const router = useRouter();

const createFuncReqStatus = funcStore.getRequestStatus("CREATE_FUNC");
const loadAssetsReqStatus = assetStore.getRequestStatus(
  "LOAD_ASSET",
  props.assetId,
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
    ? funcStore.getRequestStatus(
        "FETCH_FUNC_DETAILS",
        selectedExistingFuncId.value,
      )
    : undefined,
);

watch(selectedExistingFuncId, async (funcId) => {
  if (funcId) {
    if (!funcStore.funcDetailsById[funcId]) {
      const result = await funcStore.FETCH_FUNC_DETAILS(funcId);
      if (result.result.success) {
        selectedFuncCode.value = result.result.data.code;
      }
    } else {
      selectedFuncCode.value = funcStore.funcDetailsById[funcId]?.code ?? "";
    }
  }
});

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
    } else if ("externalProviderId" in parsed && parsed.externalProviderId) {
      return {
        type: "outputSocket",
        externalProviderId: parsed.externalProviderId,
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

const assetName = computed(() => assetStore.selectedAsset?.name ?? " none");

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

  return nameIsSet && hasOutput && existingSelected;
});

const open = (existing?: boolean, variant?: FuncKind, funcId?: FuncId) => {
  attachExisting.value = existing ?? false;

  name.value = "";
  funcKind.value = variant ?? FuncKind.Action;
  isCreate.value = false;
  isDelete.value = false;
  isRefresh.value = false;
  selectedFuncCode.value = "";
  selectedExistingFuncId.value = funcId;
  attrToValidate.value = undefined;

  attributeOutputLocationOptions.value = props.schemaVariantId
    ? funcStore
        .outputLocationOptionsForSchemaVariant(props.schemaVariantId)
        .map(({ label, value }) => ({
          label,
          value: JSON.stringify(value),
        }))
    : [];

  validationOptions.value = props.schemaVariantId
    ? funcStore.propsAsOptionsForSchemaVariant(props.schemaVariantId)
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

  let kind = ActionKind.Other;
  switch (funcKind) {
    case FuncKind.Authentication:
      return {
        type: "authenticationOptions",
        ...baseOptions,
      };
    case FuncKind.Action:
      if (isCreate.value) kind = ActionKind.Create;
      if (isDelete.value) kind = ActionKind.Delete;
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
          ...baseOptions,
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

const attachToAttributeFunction = (
  outputLocation: CreateFuncOutputLocation,
  associations: AttributeAssociations,
): AttributeAssociations => ({
  ...associations,
  prototypes: associations.prototypes.concat([
    {
      id: nilId(),
      propId: "propId" in outputLocation ? outputLocation.propId : undefined,
      externalProviderId:
        "externalProviderId" in outputLocation
          ? outputLocation.externalProviderId
          : undefined,
      prototypeArguments: [],
      componentId: undefined,
    },
  ]),
});

const reloadAssetAndRoute = async (assetId: string, funcId: string) => {
  await assetStore.LOAD_ASSET(assetId);
  close();
  router.push({
    name: "workspace-lab-assets",
    params: {
      ...router.currentRoute.value.params,
      funcId,
      assetId,
    },
  });
};

const attachExistingFunc = async () => {
  if (props.schemaVariantId && selectedExistingFuncId.value) {
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
          updatedAssociations.schemaVariantIds.push(props.schemaVariantId);
          break;
        case "attribute":
          if (attributeOutputLocationParsed.value) {
            updatedAssociations = attachToAttributeFunction(
              attributeOutputLocationParsed.value,
              associations,
            );
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
          await reloadAssetAndRoute(props.assetId, func.id);
        }
      }
    }
  }
};

const attachNewFunc = async () => {
  if (props.schemaVariantId) {
    const options = newFuncOptions(funcKind.value, props.schemaVariantId);
    const result = await funcStore.CREATE_FUNC({
      kind: funcKind.value,
      name: name.value,
      options,
    });
    if (result.result.success && props.assetId) {
      await reloadAssetAndRoute(props.assetId, result.result.data.id);
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
