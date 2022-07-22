<template>
  <div class="flex w-full h-6" @keyup.stop @keydown.stop>
    <div id="changeset-selector" class="flex">
      <div
        class="flex items-center justify-end pr-1 pt-1 text-sm text-gray-400"
      >
        changeset:
      </div>
      <div class="flex items-center mr-2">
        <SiSelect
          id="select-current-change-set"
          v-model="selectedChangeSetPk"
          class="w-32"
          value-as-number
          :options="openChangeSetsList"
          :disabled="editMode"
          size="xs"
          @change="changeSetSelected"
        />
      </div>
    </div>
    <div id="change-set-buttons" class="flex w-auto mr-2">
      <SiButton
        v-if="editMode"
        class="w-16 ml-1"
        label="cancel"
        icon="cancel"
        kind="cancel"
        size="xs"
        @click="editSessionCancel"
      />
      <SiButton
        v-if="editMode"
        class="w-16 ml-1"
        label="save"
        icon="save"
        kind="save"
        size="xs"
        @click="editSessionSave"
      />
      <SiButton
        v-if="editButtonEnabled()"
        class="w-16 ml-1"
        :class="editButtonPulse"
        label="edit"
        icon="edit"
        size="xs"
        @click="editSessionStart"
      />
      <SiButton
        v-if="applyButtonEnabled()"
        class="w-16 ml-1"
        label="apply"
        icon="merge"
        :kind="applyButtonKind"
        size="xs"
        @click="changeSetApply"
      />
    </div>

    <SiModal
      v-model="changeSetCreateModalShow"
      name="changeSetCreate"
      :esc-to-close="true"
      @before-close="changeSetCreateCancel"
    >
      <template #title>Create a changeSet</template>
      <template #body>
        <div class="flex flex-col w-full p-8">
          <SiError
            :message="changeSetCreateModalError"
            @clear="clearChangeSetCreateModalError"
          />
          <SiFormRow>
            <template #label>name:</template>
            <template #widget>
              <SiTextBox
                id="new-change-set-name"
                v-model="changeSetCreateForm.name"
                class="ml-1"
                name="new-change-set-name"
                size="sm"
                placeholder="new change set name"
                @keyup.enter="changeSetCreate"
                @keyup.escape="changeSetCreateCancel"
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
          data-cy="new-change-set-form-cancel-button"
          @click="changeSetCreateCancel"
        />
        <SiButton
          size="xs"
          label="create"
          class="ml-1"
          icon="plus"
          kind="save"
          :disabled="!changeSetCreateForm.name"
          data-cy="new-change-set-form-create-button"
          @click="changeSetCreate"
        />
      </template>
    </SiModal>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { refFrom, untilUnmounted } from "vuse-rx";

import { editButtonPulse$ } from "@/observable/change_set";
import SiSelect from "@/atoms/SiSelect.vue";
import SiButton from "@/atoms/SiButton.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiError from "@/atoms/SiError.vue";
import SiFormRow from "@/atoms/SiFormRow.vue";
// @ts-ignore const doesn't appear in index.d.ts
import { $vfm } from "vue-final-modal";
import * as Rx from "rxjs";

import { LabelList } from "@/api/sdf/dal/label_list";
import _ from "lodash";

import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";

const CHANGE_SET_NONE = -2;
const CHANGE_SET_NEW = -3;

// The selected change set primary key
const selectedChangeSetPk = ref<number>(CHANGE_SET_NONE);
// If we are in editMode or not
const editMode = refFrom<boolean>(ChangeSetService.currentEditMode());

const editButtonEnabled = () => {
  return !!(
    selectedChangeSetPk.value != CHANGE_SET_NONE &&
    selectedChangeSetPk.value != CHANGE_SET_NEW &&
    selectedChangeSetPk.value &&
    !editMode.value
  );
};
const applyButtonEnabled = () => {
  return !!(
    selectedChangeSetPk.value != CHANGE_SET_NONE &&
    selectedChangeSetPk.value != CHANGE_SET_NEW &&
    selectedChangeSetPk.value &&
    !editMode.value
  );
};
const applyButtonKind = computed(() => (editMode.value ? "standard" : "save"));

const DEFAULT_CHANGESET_LABLES = [
  { label: "- none -", value: CHANGE_SET_NONE },
  { label: ": new :", value: CHANGE_SET_NEW },
];

// The open change sets list!
const openChangeSetsList = ref<LabelList<number>>(DEFAULT_CHANGESET_LABLES);

// Update the list of open change sets dynamically
untilUnmounted(ChangeSetService.listOpenChangeSets()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
    openChangeSetsList.value = DEFAULT_CHANGESET_LABLES;
  } else {
    openChangeSetsList.value = _.concat(
      response.list,
      DEFAULT_CHANGESET_LABLES,
    );
  }
});

// Set the current change set as selected if it changes outside our POV
untilUnmounted(ChangeSetService.currentChangeSet()).subscribe((changeSet) => {
  if (changeSet) {
    selectedChangeSetPk.value = changeSet.pk;
  } else {
    selectedChangeSetPk.value = CHANGE_SET_NONE;
  }
});

const changeSetSelected = () => {
  if (selectedChangeSetPk.value == CHANGE_SET_NONE) {
    ChangeSetService.switchToHead();
  } else if (selectedChangeSetPk.value == CHANGE_SET_NEW) {
    changeSetCreateModalShow.value = true;
  } else {
    GlobalErrorService.setIfError(
      ChangeSetService.getChangeSet({
        pk: selectedChangeSetPk.value,
      }),
    );
  }
};

// Change Set Behavior
const changeSetCreateModalShow = ref(false);
const changeSetCreateModalError = ref("");
const changeSetCreateForm = ref({ name: "" });
const changeSetCreateCancel = async () => {
  selectedChangeSetPk.value = CHANGE_SET_NONE;
  changeSetCreateModalShow.value = false;
  changeSetCreateForm.value.name = "";
};
const changeSetCreate = () => {
  ChangeSetService.createChangeSet({
    changeSetName: changeSetCreateForm.value.name,
  }).subscribe(async (response) => {
    if (response.error) {
      changeSetCreateModalError.value = response.error.message;
    } else {
      await $vfm.hide("changeSetCreate");
      selectedChangeSetPk.value = response.changeSet.pk;
    }
  });
};
const changeSetApply = () => {
  ChangeSetService.applyChangeSet().subscribe((response) => {
    if (response.error) {
      GlobalErrorService.set(response);
    } else {
      selectedChangeSetPk.value = CHANGE_SET_NONE;
    }
  });
};
const clearChangeSetCreateModalError = () => {
  changeSetCreateModalError.value = "";
};
const editSessionCancel = () => {
  GlobalErrorService.setIfError(ChangeSetService.cancelEditSession());
};
const editSessionSave = () => {
  GlobalErrorService.setIfError(ChangeSetService.saveEditSession());
};
const editSessionStart = () => {
  GlobalErrorService.setIfError(
    ChangeSetService.startEditSession({
      changeSetPk: selectedChangeSetPk.value,
    }),
  );
};

const editButtonPulseStop = ref(false);
let editButtonPulseTimeout: ReturnType<typeof setTimeout>;
// This is a horrible hack to ensure we cleanup the pulse
const editButtonPulseUntil = refFrom<Date>(
  editButtonPulse$.asObservable().pipe(
    Rx.map((pulse) => {
      if (!pulse) return new Date();

      // Ensures setTimeout will trigger computed function change
      editButtonPulseStop.value = false;

      // There must be a better way of doing this with RxJs, but I've been stuck trying to find it and failing for a while, let's move on
      const until = new Date(new Date().getTime() + 5000);
      clearTimeout(editButtonPulseTimeout);
      editButtonPulseTimeout = setTimeout(() => {
        editButtonPulseStop.value = true;
      }, 5000);
      return until;
    }),
  ),
);
const editButtonPulse = computed(() => {
  let classes: Record<string, boolean> = {};
  if (
    !editButtonPulseStop.value &&
    editButtonPulseUntil.value &&
    editButtonPulseUntil.value.getTime() > new Date().getTime()
  ) {
    classes["animate-pulse"] = true;
  } else {
    classes["animate-pulse"] = false;
  }

  return classes;
});
</script>

<style lang="css" scoped>
.menu-bar {
  background-color: #212121;
}
</style>
