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
        <VButton2
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
            <VButton2
              label=""
              icon="trash"
              @click="removeSchemaVariant(svId)"
            />
          </div>
        </li>
      </ul>
      <VButton2
        :disabled="exportPkgReqStatus.isPending"
        :loading="exportPkgReqStatus.isPending"
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
import Modal from "@/ui-lib/modals/Modal.vue";
import { usePackageStore, PkgExportRequest } from "@/store/package.store";
import { useComponentsStore } from "@/store/components.store";
import VButton2 from "@/ui-lib/VButton2.vue";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import { useModal } from "@/ui-lib/modals/modal_utils";
import Stack from "@/ui-lib/layout/Stack.vue";

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
  await packageStore.EXPORT_PACKAGE({
    ...packageExportReq.value,
    schemaVariants: schemaVariantsForExport.value,
  });
  close();
  packageStore.LOAD_PACKAGES();
};
</script>
