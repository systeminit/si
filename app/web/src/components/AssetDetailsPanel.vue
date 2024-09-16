<template>
  <div class="grow relative">
    <ScrollArea v-if="editingAsset && props.schemaVariantId">
      <template #top>
        <div
          class="flex flex-row items-center justify-around gap-xs p-xs border-b dark:border-neutral-600"
        >
          <VButton
            :disabled="
              saveAssetReqStatus.isPending ||
              editingAsset.isLocked ||
              assetStore.codeSaveIsDebouncing
            "
            :loading="updateAssetReqStatus.isPending"
            :requestStatus="updateAssetReqStatus"
            icon="bolt"
            label="Regenerate Asset"
            loadingText="Regenerating Asset..."
            size="md"
            successText="Successful"
            tone="action"
            @click="executeAsset"
          />
          <VButton
            icon="clipboard-copy"
            label="Clone"
            size="md"
            tone="neutral"
            @click="() => cloneAssetModalRef?.modal?.open()"
          />
        </div>
        <AssetNameModal
          ref="cloneAssetModalRef"
          buttonLabel="Clone Asset"
          title="Asset Name"
          @submit="cloneAsset"
        />

        <ErrorMessage
          v-for="(warning, index) in assetStore.detachmentWarnings"
          :key="warning.message"
          :class="{ 'cursor-pointer': !!warning.kind }"
          class="mx-1"
          icon="alert-triangle"
          tone="warning"
          @click="openAttachModal(warning)"
        >
          {{ warning.message }}
          <VButton
            buttonRank="tertiary"
            icon="trash"
            size="xs"
            tone="destructive"
            @click.stop="assetStore.detachmentWarnings.splice(index, 1)"
          />
        </ErrorMessage>

        <AssetFuncAttachModal
          ref="attachModalRef"
          :schemaVariantId="props.schemaVariantId"
        />
      </template>

      <Stack class="p-xs" spacing="none">
        <div>
          <ErrorMessage :requestStatus="updateAssetReqStatus" variant="block" />
        </div>
        <VormInput
          id="schemaName"
          v-model="editingAsset.schemaName"
          :disabled="editingAsset.isLocked"
          compact
          label="Asset Name"
          placeholder="(mandatory) Provide the asset a name"
          type="text"
          @blur="updateAsset"
          @focus="focus"
        />

        <VormInput
          id="displayName"
          v-model="editingAsset.displayName"
          :disabled="editingAsset.isLocked"
          compact
          label="Display name"
          placeholder="(optional) Provide the asset version a display name"
          type="text"
          @blur="updateAsset"
          @focus="focus"
        />
        <VormInput
          id="category"
          v-model="editingAsset.category"
          :disabled="editingAsset.isLocked"
          compact
          label="Category"
          placeholder="(mandatory) Provide a category for the asset"
          type="text"
          @blur="updateAsset"
          @focus="focus"
        />
        <VormInput
          id="componentType"
          v-model="editingAsset.componentType"
          :disabled="editingAsset.isLocked"
          :options="componentTypeOptions"
          compact
          label="Component Type"
          type="dropdown"
          @change="updateAsset"
          @focus="focus"
        />
        <VormInput
          id="description"
          v-model="editingAsset.description"
          :disabled="editingAsset.isLocked"
          compact
          label="Description"
          placeholder="(optional) Provide a brief description of the asset"
          type="textarea"
          @blur="updateAsset"
          @focus="focus"
        />
        <VormInput
          :disabled="editingAsset.isLocked"
          compact
          label="color"
          type="container"
        >
          <ColorPicker
            id="color"
            v-model="editingAsset.color"
            @change="updateAsset"
          />
        </VormInput>

        <VormInput
          id="link"
          v-model="editingAsset.link"
          :disabled="editingAsset.isLocked"
          compact
          label="Documentation Link"
          placeholder="(optional) Provide a documentation link for the asset"
          type="url"
          @blur="updateAsset"
          @focus="focus"
        />
      </Stack>
      <template v-if="ffStore.SHOW_INTRINSIC_EDITING">
        <Stack class="p-xs" spacing="none">
          <p>Output Sockets</p>
          <ul>
            <li
              v-for="config in outputSocketIntrinsics"
              :key="config.attributePrototypeId"
            >
              <VormInput
                :id="config.socketName"
                v-model="config.value"
                :disabled="editingAsset.isLocked"
                :label="config.socketName + '&lt;&mdash;'"
                :options="optionsForIntrinsicDisplay"
                compact
                type="dropdown"
                @change="updateOutputSocketIntrinsics(config)"
              />
            </li>
          </ul>
        </Stack>
        <Stack class="p-xs" spacing="none">
          <p>Props</p>
          <ul>
            <li v-for="prop in configurableProps" :key="prop.id">
              <VormInput
                :id="prop.path"
                v-model="prop.value"
                :disabled="editingAsset.isLocked"
                :label="prop.path + '&lt;&mdash;'"
                :options="optionsForIntrinsicDisplay"
                compact
                type="dropdown"
                @change="updatePropIntrinsics(prop)"
              />
            </li>
          </ul>
        </Stack>
      </template>
    </ScrollArea>
    <div
      v-else
      class="px-2 py-sm text-center text-neutral-400 dark:text-neutral-300"
    >
      <template v-if="props.schemaVariantId"
        >Asset "{{ props.schemaVariantId }}" does not exist!
      </template>
      <template v-else>Select an asset to view its details.</template>
    </div>
    <Modal
      ref="executeAssetModalRef"
      :title="
        editingAsset && editingAsset.schemaVariantId
          ? 'Asset Updated'
          : 'New Asset Created'
      "
      size="sm"
      @closeComplete="closeHandler"
    >
      {{
        editingAsset && editingAsset.schemaVariantId
          ? "The asset you just updated will be available to use from the Assets Panel"
          : "The asset you just created will now appear in the Assets Panel."
      }}
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch, computed } from "vue";
import {
  ErrorMessage,
  Modal,
  ScrollArea,
  Stack,
  VButton,
  VormInput,
} from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import {
  FuncKind,
  FuncId,
  FuncBackendKind,
  AttributePrototypeId,
  FuncBindingKind,
  Attribute,
  AttributeArgumentBinding,
  FuncArgumentId,
} from "@/api/sdf/dal/func";
import { useAssetStore } from "@/store/asset.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  ComponentType,
  InputSocketId,
  OutputSocketId,
  SchemaVariant,
  SchemaVariantId,
  inputSocketsAndPropsFor,
} from "@/api/sdf/dal/schema";
import { useFuncStore, BindingWithBackendKind } from "@/store/func/funcs.store";
import { PropId } from "@/api/sdf/dal/prop";
import ColorPicker from "./ColorPicker.vue";
import AssetFuncAttachModal from "./AssetFuncAttachModal.vue";
import AssetNameModal from "./AssetNameModal.vue";

const props = defineProps<{
  schemaVariantId?: SchemaVariantId;
}>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();
const ffStore = useFeatureFlagsStore();
const executeAssetModalRef = ref();
const cloneAssetModalRef = ref<InstanceType<typeof AssetNameModal>>();

const focusedFormField = ref<string | undefined>();
const focus = (evt: Event) => {
  focusedFormField.value = (evt.target as HTMLInputElement).id;
};

const optionsForIntrinsicDisplay = computed(() => {
  if (!props.schemaVariantId) return [];
  const variant = assetStore.variantFromListById[props.schemaVariantId];
  if (!variant) return [];
  const opts = inputSocketsAndPropsFor(variant);
  return [...opts.propOptions, ...opts.socketOptions];
});

interface PropDisplay {
  id: PropId;
  path: string;
  value?: PropId | InputSocketId;
  attributePrototypeId?: AttributePrototypeId;
}

interface BindingWithBackendKindAndPropId extends BindingWithBackendKind {
  propId: NonNullable<PropId>;
}

const intrinsics = computed(() => {
  if (!props.schemaVariantId) return [];
  const intrinsics = funcStore.intrinsicBindingsByVariant.get(
    props.schemaVariantId,
  );
  if (!intrinsics) return [];
  return intrinsics.filter(
    (binding) => binding.backendKind === FuncBackendKind.Identity,
  );
});

const configurableProps = computed(() => {
  if (!props.schemaVariantId) return [];

  const variant = assetStore.variantFromListById[props.schemaVariantId];
  if (!variant) return [];

  const ignoreProps: PropId[] = [];
  variant?.funcIds.forEach((funcId) => {
    const summary = funcStore.funcsById[funcId];
    if (summary?.kind === FuncKind.Intrinsic) return;
    summary?.bindings.forEach((b) => {
      if (b.schemaVariantId === props.schemaVariantId)
        if ("propId" in b && b.propId) ignoreProps.push(b.propId);
    });
  });

  const _props = variant.props
    .filter((p) => p.eligibleToReceiveData)
    .filter((p) => !ignoreProps.includes(p.id));

  const propValues = {} as Record<
    PropId,
    {
      value: PropId | InputSocketId;
      attributePrototypeId: AttributePrototypeId;
    }
  >;
  intrinsics.value
    .filter(
      (binding): binding is BindingWithBackendKindAndPropId => !!binding.propId,
    )
    .forEach((binding) => {
      const arg = binding.argumentBindings.filter((a) => !!a.propId).pop();
      const inputSocket = binding.argumentBindings
        .filter((a) => !!a.inputSocketId)
        .pop();
      if (arg && arg.propId)
        propValues[binding.propId] = {
          value: `p_${arg.propId}`,
          attributePrototypeId: binding.attributePrototypeId,
        };
      if (inputSocket && inputSocket.inputSocketId)
        propValues[binding.propId] = {
          value: `s_${inputSocket.inputSocketId}`,
          attributePrototypeId: binding.attributePrototypeId,
        };
    });

  const config: PropDisplay[] = [];
  _props.forEach(({ id, path }) => {
    const vals = propValues[id];
    let value;
    let attributePrototypeId;
    if (vals) ({ value, attributePrototypeId } = vals);

    const d: PropDisplay = {
      id,
      path,
      value,
      attributePrototypeId,
    };
    config.push(d);
  });
  return config;
});

interface IntrinsicDisplay {
  attributePrototypeId: AttributePrototypeId;
  outputSocketId: OutputSocketId;
  socketName: string;
  backendKind: FuncBackendKind;
  value: InputSocketId | PropId | undefined;
}

// PSA: this is how to type guard filter so later operations know the field
// is no longer nullable b/c the filter removed any objects where the property was null
interface BindingWithBackendKindAndOutputSocket extends BindingWithBackendKind {
  outputSocketId: NonNullable<OutputSocketId>;
}

const outputSocketIntrinsics = computed(() => {
  const bindings: IntrinsicDisplay[] = [];
  intrinsics.value
    .filter(
      (binding): binding is BindingWithBackendKindAndOutputSocket =>
        !!binding.outputSocketId,
    )
    .forEach((binding) => {
      const arg = binding.argumentBindings.filter((a) => !!a.propId).pop();
      const inputSocket = binding.argumentBindings
        .filter((a) => !!a.inputSocketId)
        .pop();

      let value;
      if (arg && arg.propId) value = `p_${arg.propId}`;
      if (inputSocket && inputSocket.inputSocketId)
        value = `s_${inputSocket.inputSocketId}`;

      const d: IntrinsicDisplay = {
        value,
        attributePrototypeId: binding.attributePrototypeId,
        outputSocketId: binding.outputSocketId,
        socketName:
          assetStore.selectedSchemaVariant?.outputSockets.find(
            (s) => s.id === binding.outputSocketId,
          )?.name || "N/A",
        backendKind: binding.backendKind,
      };

      bindings.push(d);
    });
  return bindings;
});

const identityFuncId = computed(() => {
  const func = funcStore.funcList.find(
    (func) => func.kind === FuncKind.Intrinsic && func.name === "si:identity",
  );
  return func?.funcId as FuncId;
});

const identityFuncArgumentId = computed(() => {
  const func = funcStore.funcList.find(
    (func) => func.kind === FuncKind.Intrinsic && func.name === "si:identity",
  );
  return func?.arguments[0]?.id as FuncArgumentId;
});

const commonBindingConstruction = (
  data: PropDisplay | IntrinsicDisplay,
): Attribute | undefined => {
  if (!props.schemaVariantId) return;
  if (!data.value) return;

  const arg: AttributeArgumentBinding = {
    funcArgumentId: identityFuncArgumentId.value,
    attributePrototypeArgumentId: null,
    inputSocketId: null,
    propId: null,
  };
  if (data.value.startsWith("s_"))
    arg.inputSocketId = data.value.replace("s_", "");
  else if (data.value.startsWith("p_"))
    arg.propId = data.value.replace("p_", "");

  const binding: Attribute = {
    // NOTE: attributePrototypeId is null when we swap fns for a new binding, it is required when staying with the same func and switching args
    attributePrototypeId: null,
    componentId: null,
    funcId: identityFuncId.value,
    schemaVariantId: props.schemaVariantId,
    bindingKind: FuncBindingKind.Attribute,
    argumentBindings: [arg],
    propId: "id" in data ? data.id : null,
    outputSocketId: "outputSocketId" in data ? data.outputSocketId : null,
  };
  return binding;
};

const updatePropIntrinsics = async (data: PropDisplay) => {
  const binding = commonBindingConstruction(data);
  if (binding) await funcStore.CREATE_BINDING(identityFuncId.value, [binding]);
};

const updateOutputSocketIntrinsics = async (data: IntrinsicDisplay) => {
  const binding = commonBindingConstruction(data);
  if (binding) await funcStore.CREATE_BINDING(identityFuncId.value, [binding]);
};

const openAttachModal = (warning: { kind?: FuncKind; funcId?: FuncId }) => {
  if (!warning.kind) return;
  attachModalRef.value?.open(true, warning.kind, warning.funcId);
};

const componentTypeOptions = [
  { label: "Component", value: ComponentType.Component },
  {
    label: "Configuration Frame (down)",
    value: ComponentType.ConfigurationFrameDown,
  },
  {
    label: "Configuration Frame (up)",
    value: ComponentType.ConfigurationFrameUp,
  },
];

const attachModalRef = ref<InstanceType<typeof AssetFuncAttachModal>>();

const editingAsset = ref(_.cloneDeep(assetStore.selectedSchemaVariant));
watch(
  () => assetStore.selectedSchemaVariant,
  () => {
    // don't overwrite a form field that currently has focus
    const data = _.cloneDeep(assetStore.selectedSchemaVariant);
    if (!data) return;
    if (focusedFormField.value)
      delete data[focusedFormField.value as keyof SchemaVariant];
    if (editingAsset.value) Object.assign(editingAsset.value, data);
  },
  { deep: true },
);

const updateAsset = async () => {
  // this is just for blur
  focusedFormField.value = undefined;
  if (
    !editingAsset.value ||
    editingAsset.value.isLocked ||
    _.isEqual(editingAsset.value, assetStore.selectedSchemaVariant)
  )
    return;

  // const code = funcStore.funcCodeById[editingAsset.value.assetFuncId]?.code;
  // if (code)
  await assetStore.SAVE_SCHEMA_VARIANT(editingAsset.value);
  /* else
    throw new Error(
      `${editingAsset.value.assetFuncId} Func not found on Variant ${editingAsset.value.schemaVariantId}. This should not happen.`,
    ); */
};

const updateAssetReqStatus = assetStore.getRequestStatus(
  "REGENERATE_VARIANT",
  assetStore.selectedVariantId,
);
const saveAssetReqStatus = assetStore.getRequestStatus(
  "SAVE_SCHEMA_VARIANT",
  assetStore.selectedVariantId,
);
const executeAsset = async () => {
  if (editingAsset.value) {
    await assetStore.REGENERATE_VARIANT(editingAsset.value.schemaVariantId);
  }
};

const closeHandler = () => {
  assetStore.executeSchemaVariantTaskId = undefined;
};

const cloneAsset = async (name: string) => {
  if (editingAsset.value?.schemaVariantId) {
    const result = await assetStore.CLONE_VARIANT(
      editingAsset.value.schemaVariantId,
      name,
    );
    if (result.result.success) {
      assetStore.setSchemaVariantSelection(result.result.data.schemaVariantId);
      cloneAssetModalRef.value?.modal?.close();
    } else if (result.result.statusCode === 409) {
      cloneAssetModalRef.value?.setError(
        "That name is already in use, please choose another",
      );
    }
    cloneAssetModalRef.value?.reset();
  }
};
</script>
