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
          v-model="funcVariant"
          label="Kind"
          type="dropdown"
          :options="funcVariantOptions"
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
        <VormInput
          v-if="funcVariant === FuncVariant.Action && !attachExisting"
          v-model="actionKind"
          label="Action Kind"
          type="dropdown"
          :options="actionKindOptions"
        />
        <VormInput
          v-if="funcVariant === FuncVariant.Attribute"
          v-model="attributeOutputLocation"
          label="Output Location"
          type="dropdown"
          required
          :options="attributeOutputLocationOptions"
        />
        <VormInput
          v-if="funcVariant === FuncVariant.Validation"
          v-model="attrToValidate"
          label="Attribute to Validate"
          type="dropdown"
          required
          :options="validationOptions"
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
          <RequestStatusMessage
            :requestStatus="loadFuncDetailsReq.value"
            showLoaderWithoutMessage
          />
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
import { ref, computed, watch } from "vue";
import {
  Modal,
  useModal,
  VormInput,
  VButton,
  RequestStatusMessage,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import clsx from "clsx";
import uniqBy from "lodash-es/uniqBy";
import { FuncVariant, CUSTOMIZABLE_FUNC_TYPES } from "@/api/sdf/dal/func";
import { FuncId, useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import {
  CreateFuncOutputLocation,
  CreateFuncOptions,
  FuncAssociations,
  ValidationAssociations,
  AttributeAssociations,
} from "@/store/func/types";
import { ActionKind } from "@/store/fixes.store";
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

const funcVariantOptions = Object.keys(CUSTOMIZABLE_FUNC_TYPES).map(
  (variant) => ({
    label: CUSTOMIZABLE_FUNC_TYPES[variant as FuncVariant]?.singularLabel,
    value: variant as string,
  }),
);

const attachExisting = ref(false);

const name = ref("");
const funcVariant = ref(FuncVariant.Action);

const actionKind = ref(ActionKind.Other);
const actionKindOptions = Object.values(ActionKind);

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
    .filter((func) => func.variant === funcVariant.value)
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

const assetName = computed(() =>
  assetStore.selectedAssetId
    ? assetStore.assetsById[assetStore.selectedAssetId]?.name ?? "none"
    : "none",
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
    funcVariant.value !== FuncVariant.Attribute ||
    !!attributeOutputLocationParsed.value;
  const hasAttrToValidate =
    funcVariant.value !== FuncVariant.Validation || !!attrToValidate.value;
  const existingSelected =
    !attachExisting.value || !!selectedExistingFuncId.value;

  return nameIsSet && hasOutput && hasAttrToValidate && existingSelected;
});

const open = (existing?: boolean) => {
  attachExisting.value = existing ?? false;

  name.value = "";
  funcVariant.value = FuncVariant.Action;
  actionKind.value = ActionKind.Other;
  selectedExistingFuncId.value = undefined;
  selectedFuncCode.value = "";
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
  funcVariant: FuncVariant,
  schemaVariantId: string,
): CreateFuncOptions | undefined => {
  const baseOptions = {
    schemaVariantId,
  };
  switch (funcVariant) {
    case FuncVariant.Action:
      return {
        type: "actionOptions",
        actionKind: actionKind.value,
        ...baseOptions,
      };
    case FuncVariant.Attribute:
      if (attributeOutputLocationParsed.value) {
        return {
          type: "attributeOptions",
          outputLocation: attributeOutputLocationParsed.value,
          ...baseOptions,
        };
      }
      break;
    case FuncVariant.CodeGeneration:
      return {
        type: "codeGenerationOptions",
        ...baseOptions,
      };
    case FuncVariant.Confirmation:
      return {
        type: "confirmationOptions",
        ...baseOptions,
      };
    case FuncVariant.Qualification:
      return {
        type: "qualificationOptions",
        ...baseOptions,
      };
    case FuncVariant.Validation:
      if (attrToValidate.value) {
        return {
          type: "validationOptions",
          propToValidate: attrToValidate.value,
          ...baseOptions,
        };
      }
      break;
    default:
      return;
  }
};

const attachToLeafFunctionOrAction = (
  schemaVariantId: string,
  associations: FuncAssociations,
): FuncAssociations =>
  associations.type === "codeGeneration" ||
  associations.type === "confirmation" ||
  associations.type === "qualification" ||
  associations.type === "action"
    ? {
        ...associations,
        schemaVariantIds: Array.from(
          new Set(associations.schemaVariantIds.concat([schemaVariantId])),
        ),
      }
    : associations;

const attachToValidationFunction = (
  schemaVariantId: string,
  propId: string,
  associations: ValidationAssociations,
): ValidationAssociations => ({
  ...associations,
  prototypes: uniqBy(
    associations.prototypes.concat([{ schemaVariantId, propId }]),
    (proto) => proto.propId,
  ),
});

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
      let updatedAssocations: FuncAssociations | undefined;
      const associations = func?.associations;
      if (!associations) {
        return;
      }

      switch (associations.type) {
        case "action":
        case "codeGeneration":
        case "confirmation":
        case "qualification":
          updatedAssocations = attachToLeafFunctionOrAction(
            props.schemaVariantId,
            associations,
          );
          break;
        case "attribute":
          if (attributeOutputLocationParsed.value) {
            updatedAssocations = attachToAttributeFunction(
              attributeOutputLocationParsed.value,
              associations,
            );
          }
          break;
        case "validation":
          if (attrToValidate.value) {
            updatedAssocations = attachToValidationFunction(
              props.schemaVariantId,
              attrToValidate.value,
              associations,
            );
          }
          break;
        default:
          break;
      }
      if (updatedAssocations) {
        func.associations = updatedAssocations;
        const result = await funcStore.updateFuncMetadata(func);
        if (result.result.success && props.assetId) {
          await reloadAssetAndRoute(props.assetId, func.id);
        }
      }
    }
  }
};

const attachNewFunc = async () => {
  if (props.schemaVariantId) {
    const options = newFuncOptions(funcVariant.value, props.schemaVariantId);
    if (options) {
      const result = await funcStore.CREATE_FUNC({
        variant: funcVariant.value,
        name: name.value,
        options,
      });
      if (result.result.success && props.assetId) {
        await reloadAssetAndRoute(props.assetId, result.result.data.id);
      }
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
</script>
