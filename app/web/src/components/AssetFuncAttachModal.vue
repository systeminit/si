<template>
  <Modal ref="modalRef" :title="title">
    <Stack>
      <VormInput
        v-model="funcVariant"
        label="Kind"
        type="dropdown"
        :options="funcVariantOptions"
      />
      <VormInput
        v-model="name"
        label="Name"
        required
        placeholder="The name of the function"
      />
      <VormInput
        v-if="funcVariant === FuncVariant.Action"
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
        :request-status="createFuncReqStatus"
      />
      <VButton
        :loading="showLoading"
        :disabled="!nameIsSet"
        loading-text="Attaching new function..."
        label="Attach new function"
        tone="action"
        icon="plus"
        size="sm"
        @click="attachNewFunc"
      />
    </Stack>
  </Modal>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import {
  Modal,
  useModal,
  Stack,
  VormInput,
  VButton,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { FuncVariant, CUSTOMIZABLE_FUNC_TYPES } from "@/api/sdf/dal/func";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import {
  CreateFuncOutputLocation,
  CreateFuncOptions,
} from "@/store/func/types";
import { ActionKind } from "@/store/fixes.store";

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

const name = ref("");
const funcVariant = ref(FuncVariant.Action);

const actionKind = ref(ActionKind.Other);
const actionKindOptions = Object.values(ActionKind);

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

const title = computed(() =>
  assetName.value
    ? `Attach new function to ${assetName.value}`
    : `Attach new function`,
);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const nameIsSet = computed(() => name.value && name.value.length > 0);

const open = () => {
  name.value = "";
  funcVariant.value = FuncVariant.Action;
  actionKind.value = ActionKind.Other;
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
        await assetStore.LOAD_ASSET(props.assetId);
        close();
        router.push({
          name: "workspace-lab-assets",
          params: {
            ...router.currentRoute.value.params,
            funcId: result.result.data.id,
            assetId: props.assetId,
          },
        });
      }
    }
  }
};

defineExpose({ open, close });
</script>
