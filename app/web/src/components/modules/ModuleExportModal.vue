<template>
  <Modal ref="modalRef" :title="title">
    <Stack>
      <VormInput
        v-model="packageExportReq.name"
        label="Name"
        required
        placeholder="The name of this module..."
      />
      <VormInput
        v-if="!autoVersion"
        v-model="packageExportReq.version"
        label="Version"
        required
        placeholder="The version of this module..."
      />
      <VormInput
        v-model="packageExportReq.description"
        label="Description"
        type="textarea"
        placeholder="Give this module a short description..."
      />
      <div class="flex flex-row items-end gap-sm">
        <VormInput
          v-model="selectedSchemaVariant"
          label="Assets"
          type="dropdown"
          class="flex-1"
          :options="schemaVariantOptions"
        />
        <VButton
          label="Add"
          tone="action"
          icon="plus"
          size="xs"
          class="mb-1"
          @click="addSchemaVariantToExport"
        />
      </div>
      <ul class="flex flex-col gap-2xs">
        <li
          v-for="svId in schemaVariantsForExport"
          :key="svId"
          class="flex px-1 items-center"
        >
          <span class="pr-2" role="decoration">•</span>
          {{ schemaVariantsById?.[svId]?.schemaName }}
          <VButton
            class="ml-auto"
            size="xs"
            icon="trash"
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
          href="https://www.apache.org/licenses/LICENSE-2.0"
          class="text-green-500"
          >Apache License, Version 2.0</a
        >, and that you intend for System Initiative, Inc. to distribute it.
      </p>
      <VButton
        :requestStatus="exportPkgReqStatus"
        :loadingText="loadingText"
        :disabled="!enableExportButton"
        :label="label"
        tone="action"
        icon="cloud-upload"
        size="sm"
        @click="exportPkg"
      />
    </Stack>
  </Modal>
</template>

<script lang="ts" setup>
import { emit } from "process";
import { ref, computed } from "vue";
import {
  Modal,
  VButton,
  VormInput,
  useModal,
  Stack,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { format as dateFormat } from "date-fns";
import { useComponentsStore } from "@/store/components.store";
import { useModuleStore, PkgExportRequest } from "@/store/module.store";

const moduleStore = useModuleStore();
const componentStore = useComponentsStore();
const modalRef = ref<InstanceType<typeof Modal>>();
const exportPkgReqStatus = moduleStore.getRequestStatus("EXPORT_MODULE");

const props = withDefaults(
  defineProps<{
    title?: string;
    label?: string;
    loadingText?: string;
    autoVersion?: boolean;
    preSelectedSchemaVariantId?: string;
  }>(),
  {
    title: "Export Module",
    label: "Export",
    loadingText: "Exporting...",
    autoVersion: false,
  },
);

const emits = defineEmits(["exportSuccess"]);

const emptyExportPackageReq: PkgExportRequest = {
  name: "",
  description: undefined,
  version: "",
  schemaVariants: [],
};

const selectedSchemaVariant = ref();
const schemaVariantsForExport = ref<string[]>([]);

const packageExportReq = ref<PkgExportRequest>({ ...emptyExportPackageReq });

const addSchemaVariantToExport = () => {
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

defineExpose({ open, close });

const schemaVariantsById = computed(() => componentStore.schemaVariantsById);

const schemaVariantOptions = computed(() =>
  componentStore.schemaVariants
    .filter((sv) => !schemaVariantsForExport.value.includes(sv.id))
    .map((sv) => ({
      label: sv.schemaName,
      value: sv.id,
    })),
);

const getVersionTimestamp = () => dateFormat(Date.now(), "yyyyMMddkkmmss");

const enableExportButton = computed(() => {
  if (packageExportReq.value?.name?.trim().length === 0) {
    return false;
  }
  if (
    !props.autoVersion &&
    packageExportReq.value?.version?.trim().length === 0
  ) {
    return false;
  }
  if (schemaVariantsForExport.value?.length === 0) {
    return false;
  }

  return true;
});

const exportPkg = async () => {
  if (props.autoVersion) {
    packageExportReq.value.version = getVersionTimestamp();
  }
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
