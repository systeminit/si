<template>
  <div class="flex w-full h-6" @keyup.stop @keydown.stop>
    <div id="system-selector" class="flex">
      <div
        class="flex items-center justify-end pr-1 pt-1 text-sm text-gray-400"
      >
        system:
      </div>
      <div class="flex items-center mr-2">
        <SiSelect
          id="select-current-system"
          v-model="selectedSystemId"
          value-as-number
          :options="systemsList"
          :disabled="false"
          class="w-32"
          @change="systemSelected"
        />
      </div>
    </div>

    <SiModal
      v-model="systemCreateModalShow"
      name="systemCreate"
      :esc-to-close="true"
      @before-close="systemCreateCancel"
    >
      <template #title>Create a system</template>
      <template #body>
        <div class="flex flex-col w-full p-8">
          <SiError
            :message="systemCreateModalError"
            @clear="clearSystemCreateModalError"
          />
          <SiFormRow>
            <template #label>name:</template>
            <template #widget>
              <SiTextBox
                id="new-system-name"
                v-model="systemCreateForm.name"
                class="ml-1"
                name="new-system-name"
                size="sm"
                placeholder="new system name"
                @keyup.enter="systemCreate"
                @keyup.escape="systemCreateCancel"
              />
            </template>
          </SiFormRow>
        </div>
      </template>
      <template #buttons>
        <SiButton
          size="xs"
          label="cancel"
          icon="cancel"
          kind="cancel"
          data-cy="new-system-form-cancel-button"
          @click="systemCreateCancel"
        />
        <SiButton
          size="xs"
          label="create"
          class="ml-1"
          icon="plus"
          kind="save"
          :disabled="!systemCreateForm.name"
          data-cy="new-system-form-create-button"
          @click="systemCreate"
        />
      </template>
    </SiModal>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

import { LabelList } from "@/api/sdf/dal/label_list";
import SiButton from "@/atoms/SiButton.vue";
import SiError from "@/atoms/SiError.vue";
import SiFormRow from "@/atoms/SiFormRow.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { GlobalErrorService } from "@/service/global_error";
import { ChangeSetService } from "@/service/change_set";
import { SystemService } from "@/service/system";
import _ from "lodash";
// @ts-ignore const doesn't appear in index.d.ts
import { $vfm } from "vue-final-modal";
import { refFrom, untilUnmounted } from "vuse-rx";

const SYSTEM_NONE = -1;
const SYSTEM_NEW = -3;

// The selected system id
const selectedSystemId = ref<number>(SYSTEM_NONE);

const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

const defaultSystemLabels = computed(() => {
  const labels = [{ label: "- none -", value: SYSTEM_NONE }];
  if (!(editMode.value === undefined || editMode.value == false)) {
    labels.push({ label: ": new :", value: SYSTEM_NEW });
  }
  return labels;
});

// The list of systems
const systemsList = ref<LabelList<number>>(defaultSystemLabels.value);

const systemSelected = () => {
  if (selectedSystemId.value == SYSTEM_NONE) {
    SystemService.switchToNone();
  } else if (selectedSystemId.value == SYSTEM_NEW) {
    systemCreateModalShow.value = true;
  } else {
    GlobalErrorService.setIfError(
      SystemService.getSystem({
        systemId: selectedSystemId.value,
      }),
    );
  }
};

const systemCreateModalShow = ref(false);
const systemCreateModalError = ref("");
const systemCreateForm = ref({ name: "" });
const systemCreateCancel = async () => {
  selectedSystemId.value = SYSTEM_NONE;
  systemCreateModalShow.value = false;
  systemCreateForm.value.name = "";
};
const systemCreate = () => {
  SystemService.createSystem({
    name: systemCreateForm.value.name,
  }).subscribe(async (response) => {
    if (response.error) {
      systemCreateModalError.value = response.error.message;
    } else {
      await $vfm.hide("systemCreate");
      selectedSystemId.value = response.system.id;
      SystemService.switchTo(response.system);
    }
  });
};
const clearSystemCreateModalError = () => {
  systemCreateModalError.value = "";
};

// Update the list of systems dynamically
untilUnmounted(SystemService.listSystems()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
    systemsList.value = defaultSystemLabels.value;
  } else {
    systemsList.value = _.concat(response.list, defaultSystemLabels.value);
  }
});

// Set the currentsystem as selected if it changes outside our POV
untilUnmounted(SystemService.currentSystem()).subscribe((system) => {
  if (system) {
    selectedSystemId.value = system.id;
  } else {
    selectedSystemId.value = SYSTEM_NONE;
  }
});
</script>

<style lang="css" scoped>
.menu-bar {
  background-color: #212121;
}
</style>
