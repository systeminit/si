<template>
  <Modal ref="modalRef" title="Export package">
    <Stack>
      <VormInput
        v-model="packageExportReq.name"
        label="Name"
        required
        placeholder="The name of this package..."
      />
      <VormInput
        v-model="packageExportReq.version"
        label="Version"
        required
        placeholder="The version of this package..."
      />
      <VormInput
        v-model="packageExportReq.description"
        label="Description"
        type="textarea"
        placeholder="Give this package a short description..."
      />
      <div class="flex flex-row items-end gap-sm">
        <VormInput
          v-model="selectedSchemaVariant"
          label="Schema Variants"
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
      <ul>
        <li
          v-for="svId in schemaVariantsForExport"
          :key="svId"
          class="flex flex-row gap-sm px-1"
        >
          <div class="pr-2" role="decoration">•</div>
          {{ schemaVariantsById?.[svId]?.schemaName }}
          <div class="ml-auto">
            <VButton
              label=""
              icon="trash"
              @click="removeSchemaVariant(svId)"
            />
          </div>
        </li>
      </ul>
      <ErrorMessage
        v-if="exportPkgReqStatus.isError"
        :request-status="exportPkgReqStatus"
      />
      <VButton
        :request-status="exportPkgReqStatus"
        loading-text="Exporting..."
        label="Export"
        tone="action"
        icon="plus"
        size="sm"
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
import { usePackageStore, PkgExportRequest } from "@/store/package.store";
import { useComponentsStore } from "@/store/components.store";

const packageStore = usePackageStore();
const componentStore = useComponentsStore();
const modalRef = ref<InstanceType<typeof Modal>>();
const exportPkgReqStatus = packageStore.getRequestStatus("EXPORT_PACKAGE");

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

const exportPkg = async () => {
  const result = await packageStore.EXPORT_PACKAGE({
    ...packageExportReq.value,
    schemaVariants: schemaVariantsForExport.value,
  });
  if (result.result.success) {
    close();
    await packageStore.LOAD_PACKAGES();
  }
};
</script>
