<template>
  <Modal ref="modalRef" :title="title">
    <Stack>
      <VormInput
        v-model="packageExportReq.name"
        label="Name"
        placeholder="The name of this module..."
        required
      />
      <VormInput
        v-model="packageExportReq.description"
        label="Description"
        placeholder="Give this module a short description..."
        type="textarea"
      />
      <div class="flex flex-row items-end gap-sm">
        <VormInput
          v-model="selectedSchemaVariant"
          :options="schemaVariantOptions"
          class="flex-1"
          label="Assets"
          type="dropdown"
        />
        <VButton
          class="mb-1"
          icon="plus"
          label="Add"
          size="xs"
          tone="action"
          @click="addSchemaVariantToExport"
        />
      </div>
      <ul class="flex flex-col gap-2xs max-h-72">
        <li
          v-for="svId in schemaVariantsForExport"
          :key="svId"
          class="flex px-1 items-center"
        >
          <span class="pr-2 select-none">•</span>
          {{ schemaVariantsById?.[svId]?.schemaName }}
          <span class="text-2xs italic text-neutral-500 ml-xs">
            {{ schemaVariantsById?.[svId]?.displayName }}
            {{ svTimestampStringById[svId] }}
          </span>
          <VButton
            class="ml-auto"
            icon="trash"
            size="xs"
            @click="removeSchemaVariant(svId)"
          />
        </li>
      </ul>
      <ErrorMessage
        v-if="exportPkgReqStatus.isError"
        :requestStatus="exportPkgReqStatus"
      />
      <p>
        Everything you contribute will receive a code review, and we will reach
        out if we have any questions or concerns. Assuming things look good, we
        will then include your asset in a future version of System Initiative!
      </p>
      <p>
        By clicking the 'Contribute to System Initiative' button, you agree to
        license any code submitted under the terms of the
        <a
          class="text-green-500"
          href="https://www.apache.org/licenses/LICENSE-2.0"
          >Apache License, Version 2.0</a
        >, and that you intend for System Initiative, Inc. to distribute it.
      </p>
      <VButton
        :disabled="!enableExportButton"
        :label="label"
        :loadingText="loadingText"
        :requestStatus="exportPkgReqStatus"
        icon="cloud-upload"
        size="sm"
        tone="action"
        @click="exportPkg"
      />
    </Stack>
  </Modal>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import {
  Modal,
  VButton,
  VormInput,
  useModal,
  Stack,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { format as dateFormat, parseISO } from "date-fns";
import * as _ from "lodash-es";
import { useComponentsStore } from "@/store/components.store";
import { useModuleStore, ModuleExportRequest } from "@/store/module.store";

const moduleStore = useModuleStore();
const componentStore = useComponentsStore();
const modalRef = ref<InstanceType<typeof Modal>>();
const exportPkgReqStatus = moduleStore.getRequestStatus("EXPORT_MODULE");

const props = withDefaults(
  defineProps<{
    title?: string;
    label?: string;
    loadingText?: string;
    preSelectedSchemaVariantId?: string;
  }>(),
  {
    title: "Export Module",
    label: "Export",
    loadingText: "Exporting...",
  },
);

const emits = defineEmits(["exportSuccess"]);

const emptyExportPackageReq: ModuleExportRequest = {
  name: "",
  description: undefined,
  version: "",
  schemaVariants: [],
};

const selectedSchemaVariant = ref();
const schemaVariantsForExport = ref<string[]>([]);

const packageExportReq = ref<ModuleExportRequest>({ ...emptyExportPackageReq });

const addSchemaVariantToExport = () => {
  if (!selectedSchemaVariant.value) {
    return;
  }
  schemaVariantsForExport.value.push(selectedSchemaVariant.value);
  selectedSchemaVariant.value = undefined;
};

const removeSchemaVariant = (idToRemove: string) => {
  schemaVariantsForExport.value = schemaVariantsForExport.value.filter(
    (svId) => svId !== idToRemove,
  );
};

const { open: openModal, close } = useModal(modalRef);
const open = () => {
  selectedSchemaVariant.value = undefined;
  schemaVariantsForExport.value = [];
  if (props.preSelectedSchemaVariantId) {
    schemaVariantsForExport.value = [props.preSelectedSchemaVariantId];
  }
  packageExportReq.value = { ...emptyExportPackageReq };
  openModal();
};

const isOpen = computed(() => modalRef.value?.isOpen);

defineExpose({ open, close, isOpen });

const schemaVariantsById = computed(() => componentStore.schemaVariantsById);

const svTimestampStringById = computed(() =>
  _.reduce(
    componentStore.schemaVariantsById,
    (acc, sv, id) => ({
      ...acc,
      [id]: dateFormat(parseISO(sv.created_at), "M/d/y h:mm:ss a"),
    }),
    {} as Record<string, string>,
  ),
);

const schemaVariantOptions = computed(() =>
  componentStore.schemaVariants
    .filter((sv) => !schemaVariantsForExport.value.includes(sv.schemaVariantId))
    // .filter((sv) => sv.isDefault)
    .map((sv) => ({
      label: `${sv.schemaName}: ${sv.displayName} ${
        svTimestampStringById.value[sv.schemaVariantId]
      }`,
      value: sv.schemaVariantId,
    })),
);

const getVersionTimestamp = () => dateFormat(Date.now(), "yyyyMMddkkmmss");

const enableExportButton = computed(() => {
  return packageExportReq.value?.name?.trim().length !== 0;
});

const exportPkg = async () => {
  packageExportReq.value.version = getVersionTimestamp();
  const result = await moduleStore.EXPORT_MODULE({
    ...packageExportReq.value,
    schemaVariants: schemaVariantsForExport.value,
  });
  if (result.result.success) {
    emits("exportSuccess");
    close();
    await moduleStore.LOAD_LOCAL_MODULES();
  }
};
</script>
