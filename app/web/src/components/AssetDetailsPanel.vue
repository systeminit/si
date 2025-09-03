<template>
  <div class="grow relative">
    <ScrollArea v-if="editingAsset && schemaVariantId" ref="scrollAreaRef">
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
          instructions="(mandatory) Provide the asset a name"
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
          instructions="(optional) Provide the asset version a display name"
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
          instructions="(mandatory) Provide a category for the asset"
          type="text"
          @blur="updateAsset"
          @focus="focus"
        />
        <VormInput
          id="componentType"
          disabled
          compact
          label="Component Type"
          type="text"
          :modelValue="'Component'"
        />
        <VormInput
          id="description"
          v-model="editingAsset.description"
          :disabled="editingAsset.isLocked"
          compact
          label="Description"
          instructions="(optional) Provide a brief description of the asset"
          type="textarea"
          @blur="updateAsset"
          @focus="focus"
        />
        <VormInput
          :disabled="editingAsset.isLocked"
          compact
          label="Color"
          type="container"
        >
          <ColorPicker
            id="color"
            v-model="editingAsset.color"
            :parentElement="scrollAreaRef?.scrollElement || undefined"
            :disabled="editingAsset.isLocked"
            @change="updateAsset"
          />
        </VormInput>
        <VormInput
          id="link"
          v-model="editingAsset.link"
          :disabled="editingAsset.isLocked"
          compact
          label="Documentation Link"
          instructions="(optional) Provide a documentation link for the asset"
          type="url"
          @blur="updateAsset"
          @focus="focus"
        />
      </Stack>
      <Stack v-if="funcListRequest.isPending" class="p-xs" spacing="none">
        <div class="flex items-center">
          <span class="uppercase font-bold py-3"
            >CONFIGURE DATA PROPAGATION</span
          >
          <Icon class="ml-xs" size="lg" name="loader" />
        </div>
      </Stack>
      <div v-else>
        <Stack class="p-xs" spacing="none">
          <span class="uppercase font-bold py-3"
            >CONFIGURE DATA PROPAGATION</span
          >
          <div class="text-xs pb-xs">
            <div class="flex items-center">
              <p>Choose how output sockets and props get their values.</p>
              <IconButton
                class="ml-xs"
                :icon="
                  showIntrinsicFuncOptionsText
                    ? 'chevron--down'
                    : 'chevron--right'
                "
                iconIdleTone="neutral"
                size="sm"
                @click="toggleIntrinsicFuncOptionsText"
              />
            </div>
            <ul
              v-if="showIntrinsicFuncOptionsText"
              class="list-disc pl-md py-xs"
            >
              <li><strong>Unset:</strong> the value is unset</li>
              <li>
                <strong>Identity:</strong> get the value from another prop or
                input socket without modifying it
              </li>
              <li>
                <strong>Normalize to Array:</strong> get the value from another
                prop or input socket, but modify it if the input is empty
                (becomes "[]") or not an array (wrapped with "[]")
              </li>
            </ul>
          </div>
          <span class="uppercase font-bold text-sm">Output Sockets</span>
          <ul v-if="outputSocketIntrinsics.length > 0">
            <li
              v-for="config in outputSocketIntrinsics"
              :key="config.attributePrototypeId"
            >
              <AssetDetailIntrinsicInput
                :schemaVariantId="schemaVariantId"
                :isLocked="editingAsset.isLocked"
                :data="config"
                @change="updateOutputSocketIntrinsics"
                @changeIntrinsicFunc="changeIntrinsicFunc"
              />
            </li>
          </ul>
          <p v-else class="text-xs pb-4 pt-2">
            No output sockets exist for asset.
          </p>
        </Stack>
        <Stack class="p-xs" spacing="none">
          <span class="uppercase font-bold text-sm">Props</span>
          <ul v-if="configurableProps.length > 0">
            <li v-for="prop in configurableProps" :key="prop.id">
              <AssetDetailIntrinsicInput
                :schemaVariantId="schemaVariantId"
                :isLocked="editingAsset.isLocked"
                :data="prop"
                @change="updatePropIntrinsics"
                @changeIntrinsicFunc="changeIntrinsicFunc"
              />
            </li>
          </ul>
          <p v-else class="text-xs pb-4 pt-2">No props exist for asset.</p>
        </Stack>
      </div>
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
import { ref, watch, computed, toRaw } from "vue";
import {
  ErrorMessage,
  Modal,
  ScrollArea,
  Stack,
  VButton,
  VormInput,
  Icon,
  IconButton,
  ColorPicker,
} from "@si/vue-lib/design-system";
import * as _ from "lodash-es";
import { useToast } from "vue-toastification";
import {
  FuncKind,
  FuncId,
  FuncBackendKind,
  AttributePrototypeId,
  FuncBindingKind,
  Attribute,
  AttributeArgumentBinding,
  FuncArgumentId,
  IntrinsicDisplay,
  PropDisplay,
  BindingWithBackendKindAndPropId,
  BindingWithBackendKindAndOutputSocket,
} from "@/api/sdf/dal/func";
import { useAssetStore } from "@/store/asset.store";
import {
  InputSocketId,
  SchemaVariant,
  SchemaVariantId,
} from "@/api/sdf/dal/schema";
import { INTRINSICS_DISPLAYED, useFuncStore } from "@/store/func/funcs.store";
import { PropId } from "@/api/sdf/dal/prop";
import AssetFuncAttachModal from "./AssetFuncAttachModal.vue";
import AssetNameModal from "./AssetNameModal.vue";
import AssetDetailIntrinsicInput from "./AssetDetailIntrinsicInput.vue";

const toast = useToast();

const props = defineProps<{
  schemaVariantId?: SchemaVariantId;
}>();

const assetStore = useAssetStore();
const funcStore = useFuncStore();
const executeAssetModalRef = ref();
const cloneAssetModalRef = ref<InstanceType<typeof AssetNameModal>>();
const scrollAreaRef = ref<InstanceType<typeof ScrollArea>>();

const showIntrinsicFuncOptionsText = ref(false);
const toggleIntrinsicFuncOptionsText = () => {
  showIntrinsicFuncOptionsText.value = !showIntrinsicFuncOptionsText.value;
};

// if func list is loading, its because we dont have the right data
// and we dont want to display incorrect intrinsic data
const funcListRequest = funcStore.getRequestStatus("FETCH_FUNC_LIST");

const focusedFormField = ref<string | undefined>();
const focus = (evt: Event) => {
  focusedFormField.value = (evt.target as HTMLInputElement).id;
};

const unsetFuncId = computed(() => {
  const func = funcStore.funcList.find(
    (func) => func.kind === FuncKind.Intrinsic && func.name === "si:unset",
  );
  return func?.funcId as FuncId;
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
const normalizeToArrayFuncId = computed(() => {
  const func = funcStore.funcList.find(
    (func) =>
      func.kind === FuncKind.Intrinsic && func.name === "si:normalizeToArray",
  );
  return func?.funcId as FuncId;
});
const normalizeToArrayFuncArgumentId = computed(() => {
  const func = funcStore.funcList.find(
    (func) =>
      func.kind === FuncKind.Intrinsic && func.name === "si:normalizeToArray",
  );
  return func?.arguments[0]?.id as FuncArgumentId;
});

const intrinsics = computed(() => {
  if (!props.schemaVariantId) return [];
  const intrinsics = funcStore.intrinsicBindingsByVariant.get(
    props.schemaVariantId,
  );
  if (!intrinsics) return [];
  return intrinsics;
});

const _configurableProps = computed(() => {
  if (!props.schemaVariantId) return [];

  const variant = assetStore.variantFromListById[props.schemaVariantId];
  if (!variant) return [];

  const ignoreProps: PropId[] = [];
  variant?.funcIds.forEach((funcId) => {
    const summary = funcStore.funcsById[funcId];
    if (summary?.kind === FuncKind.Intrinsic)
      if (INTRINSICS_DISPLAYED.includes(summary.backendKind)) return; // ignore set string, etc
    summary?.bindings.forEach((b) => {
      if (b.schemaVariantId === props.schemaVariantId) {
        if ("propId" in b && b.propId) ignoreProps.push(b.propId);
      }
    });
  });

  const _props = variant.props
    .filter((p) => p.eligibleToReceiveData)
    .filter((p) => !p.hidden)
    .filter((p) => !ignoreProps.includes(p.id));

  const propValues = {} as Record<
    PropId,
    {
      value: PropId | InputSocketId;
      attributePrototypeId: AttributePrototypeId;
      backendKind: FuncBackendKind;
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
          backendKind: binding.backendKind,
        };
      if (inputSocket && inputSocket.inputSocketId)
        propValues[binding.propId] = {
          value: `s_${inputSocket.inputSocketId}`,
          attributePrototypeId: binding.attributePrototypeId,
          backendKind: binding.backendKind,
        };
    });

  const config: PropDisplay[] = [];
  _props.forEach(({ id, path, name }) => {
    const vals = propValues[id];
    let value;
    let attributePrototypeId;
    let backendKind;
    if (vals) ({ value, attributePrototypeId, backendKind } = vals);

    let funcId = unsetFuncId.value;
    if (backendKind === FuncBackendKind.Identity) funcId = identityFuncId.value;
    else if (backendKind === FuncBackendKind.NormalizeToArray)
      funcId = normalizeToArrayFuncId.value;

    const d: PropDisplay = {
      id,
      path,
      name,
      value,
      attributePrototypeId,
      funcId,
    };
    config.push(d);
  });
  config.sort((a, b) => a.path.localeCompare(b.path));
  return config;
});

const configurableProps = ref<PropDisplay[]>([]);
watch(
  _configurableProps,
  () => {
    configurableProps.value = toRaw(_configurableProps.value);
  },
  { immediate: true },
);

const _outputSocketIntrinsics = computed(() => {
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

      const socketName =
        assetStore.selectedSchemaVariant?.outputSockets.find(
          (s) => s.id === binding.outputSocketId,
        )?.name || "N/A";

      let value;
      if (arg && arg.propId) value = `p_${arg.propId}`;
      if (inputSocket && inputSocket.inputSocketId)
        value = `s_${inputSocket.inputSocketId}`;

      let funcId = unsetFuncId.value;
      if (binding.backendKind === FuncBackendKind.Identity)
        funcId = identityFuncId.value;
      else if (binding.backendKind === FuncBackendKind.NormalizeToArray)
        funcId = normalizeToArrayFuncId.value;

      const d: IntrinsicDisplay = {
        value,
        attributePrototypeId: binding.attributePrototypeId,
        outputSocketId: binding.outputSocketId,
        backendKind: binding.backendKind,
        socketName,
        funcId,
      };

      bindings.push(d);
    });
  bindings.sort((a, b) => a.socketName.localeCompare(b.socketName));
  return bindings;
});

const outputSocketIntrinsics = ref<IntrinsicDisplay[]>([]);
watch(
  _outputSocketIntrinsics,
  () => {
    outputSocketIntrinsics.value = toRaw(_outputSocketIntrinsics.value);
  },
  { immediate: true },
);

const commonBindingConstruction = (
  data: PropDisplay | IntrinsicDisplay,
): Attribute | undefined => {
  if (!props.schemaVariantId) return;

  // unset has no value
  if (!data.value && data.funcId !== unsetFuncId.value) return;

  let funcArgumentId = identityFuncArgumentId.value;
  if (data.funcId === normalizeToArrayFuncId.value)
    funcArgumentId = normalizeToArrayFuncArgumentId.value;

  const argumentBindings: AttributeArgumentBinding[] = [];

  if (
    data.funcId === identityFuncId.value ||
    data.funcId === normalizeToArrayFuncId.value
  ) {
    const arg: AttributeArgumentBinding = {
      funcArgumentId,
      attributePrototypeArgumentId: null,
      inputSocketId: null,
      propId: null,
    };

    if (data.value)
      if (data.value.startsWith("s_"))
        arg.inputSocketId = data.value.replace("s_", "");
      else if (data.value.startsWith("p_"))
        arg.propId = data.value.replace("p_", "");

    argumentBindings.push(arg);
  }

  const binding: Attribute = {
    // NOTE: attributePrototypeId is null when we swap fns for a new binding,
    // it is required when staying with the same func and switching args
    attributePrototypeId: data.attributePrototypeId || null,
    componentId: null,
    funcId: data.funcId,
    schemaVariantId: props.schemaVariantId,
    bindingKind: FuncBindingKind.Attribute,
    argumentBindings,
    propId: "id" in data ? data.id : null,
    outputSocketId: "outputSocketId" in data ? data.outputSocketId : null,
  };
  return binding;
};

const updatePropIntrinsics = async (data: PropDisplay) => {
  const binding = commonBindingConstruction(data);
  if (binding) {
    const resp = await funcStore.CREATE_BINDING(data.funcId, [binding]);
    if (!resp.result.success) {
      if (resp.result.statusCode === 422) {
        toast(
          "Error: chosen prop configuration is invalid. It would cause a cycle",
        );
        configurableProps.value = toRaw(_configurableProps.value);
      }
    }
  }
};

const updateOutputSocketIntrinsics = async (data: IntrinsicDisplay) => {
  const binding = commonBindingConstruction(data);
  if (binding) {
    const resp = await funcStore.CREATE_BINDING(data.funcId, [binding]);
    if (!resp.result.success) {
      if (resp.result.statusCode === 422) {
        toast(
          "Error: chosen socket configuration is invalid. It would cause a cycle",
        );
        outputSocketIntrinsics.value = toRaw(_outputSocketIntrinsics.value);
      }
    }
  }
};

const changeIntrinsicFunc = (
  intrinsicFunc: "unset" | "identity" | "normalizeToArray",
  config: PropDisplay | IntrinsicDisplay | undefined,
) => {
  if (!config) return;
  config.attributePrototypeId = undefined;
  config.value = undefined;

  // Set the func ID no matter what the rest of the config looks like.
  if (intrinsicFunc === "unset") {
    config.funcId = unsetFuncId.value;
  } else if (intrinsicFunc === "normalizeToArray") {
    config.funcId = normalizeToArrayFuncId.value;
  } else {
    config.funcId = identityFuncId.value;
  }

  // Handle updating based on the selected func and backend kind.
  if (intrinsicFunc === "unset") {
    if ("backendKind" in config) {
      config.backendKind = FuncBackendKind.Unset;
      updateOutputSocketIntrinsics(config);
    } else {
      updatePropIntrinsics(config);
    }
  } else {
    if ("backendKind" in config) {
      if (intrinsicFunc === "normalizeToArray") {
        config.backendKind = FuncBackendKind.NormalizeToArray;
      } else {
        config.backendKind = FuncBackendKind.Identity;
      }

      if (config.value) {
        updateOutputSocketIntrinsics(config);
      }
    } else if (config.value) {
      updatePropIntrinsics(config);
    }
  }
};

const openAttachModal = (warning: { kind?: FuncKind; funcId?: FuncId }) => {
  if (!warning.kind) return;
  attachModalRef.value?.open(true, warning.kind, warning.funcId);
};

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
    else editingAsset.value = data;
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
      cloneAssetModalRef.value?.modal?.close();
    } else if (!result.result.success && result.result.statusCode === 409) {
      cloneAssetModalRef.value?.setError(
        "That name is already in use, please choose another",
      );
    }
    cloneAssetModalRef.value?.reset();
  }
};
</script>
