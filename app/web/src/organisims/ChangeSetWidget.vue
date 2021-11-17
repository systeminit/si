<template>
  <div class="flex w-full h-6" @keyup.stop @keydown.stop>
    <div id="changeset-selector" class="flex">
      <div class="flex items-center justify-end pr-1 text-xs text-gray-400">
        changeset:
      </div>
      <div class="flex items-center mr-2">
        <SiSelect
          id="selectCurrentChangeSet"
          v-model="selectedChangeSetPk"
          value-as-number
          :options="openChangeSetsList"
          :disabled="editMode"
          size="xs"
          :styling="changeSetSelectorStyling()"
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
            @clear="changeSetCreateModalError.value = ''"
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
import {
  changeSet$,
  changeSetsOpenList$,
  revision$,
} from "@/observable/change_set";

import { globalErrorMessage$ } from "@/observable/global";
import { editMode$ } from "@/observable/edit_mode";

import SiSelect from "@/atoms/SiSelect.vue";
import SiButton from "@/atoms/SiButton.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiError from "@/atoms/SiError.vue";
import SiFormRow from "@/atoms/SiFormRow.vue";
// @ts-ignore
import { $vfm } from "vue-final-modal";

import { ChangeSetService } from "@/service/change_set";
import { LabelList } from "@/api/sdf/dal/label_list";
import { ChangeSet } from "@/api/sdf/dal/change_set";

import { tap } from "rxjs";
import _ from "lodash";

const CHANGE_SET_NONE = -2;
const CHANGE_SET_NEW = -3;

// The selected change set primary key
const selectedChangeSetPk = ref<number>(CHANGE_SET_NONE);
// If we are in editMode or not
const editMode = refFrom<boolean>(editMode$);
// The currently selected revision
const revision = refFrom<ChangeSet | null>(revision$);

// Styling for the change set selector and buttons
const changeSetSelectorStyling = () => {
  let classes: Record<string, any> = {};
  classes["bg-selector1"] = true;
  classes["text-gray-400"] = true;
  classes["border-gray-700"] = true;
  return classes;
};
const editButtonEnabled = () => {
  return !!(
    selectedChangeSetPk.value != CHANGE_SET_NONE &&
    selectedChangeSetPk.value != CHANGE_SET_NEW &&
    selectedChangeSetPk.value &&
    !editMode.value &&
    !revision.value
  );
};
const applyButtonEnabled = () => {
  return !!(
    selectedChangeSetPk.value != CHANGE_SET_NONE &&
    selectedChangeSetPk.value != CHANGE_SET_NEW &&
    selectedChangeSetPk.value &&
    !editMode.value &&
    !revision.value
  );
};
const applyButtonKind = computed(() => (editMode.value ? "standard" : "save"));

// The open change sets list!
const openChangeSetsList = ref<LabelList<number>>([
  { label: "- none -", value: CHANGE_SET_NONE },
  { label: ": new :", value: CHANGE_SET_NEW },
]);
const _openChangeSetsList$ = refFrom(
  changeSetsOpenList$.pipe(
    tap((response) => {
      const always = [
        { label: "- none -", value: CHANGE_SET_NONE },
        { label: ": new :", value: CHANGE_SET_NEW },
      ];
      if (response.error) {
        globalErrorMessage$.next(response);
        openChangeSetsList.value = always;
      } else {
        openChangeSetsList.value = _.concat(response.list, always);
      }
    }),
  ),
);

// Setting the selected Change Set
const _setSelectedChangeSetPk$ = refFrom(
  untilUnmounted(
    changeSet$.pipe(
      tap((changeSet) => {
        if (changeSet) {
          selectedChangeSetPk.value = changeSet.pk;
        } else {
          selectedChangeSetPk.value = CHANGE_SET_NONE;
        }
      }),
    ),
  ),
);
const changeSetSelected = async () => {
  if (selectedChangeSetPk.value == CHANGE_SET_NONE) {
    ChangeSetService.switchToHead();
  } else if (selectedChangeSetPk.value == CHANGE_SET_NEW) {
    await $vfm.show("changeSetCreate");
  } else {
    let response = await ChangeSetService.getChangeSet({
      pk: selectedChangeSetPk.value,
    });
    if (response.error) {
      globalErrorMessage$.next(response);
    }
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
const changeSetCreate = async () => {
  const response = await ChangeSetService.createChangeSet({
    changeSetName: changeSetCreateForm.value.name,
  });
  if (response.error) {
    changeSetCreateModalError.value = response.error.message;
  } else {
    await $vfm.hide("changeSetCreate");
    selectedChangeSetPk.value = response.changeSet.pk;
  }
};
const changeSetApply = async () => {
  let response = await ChangeSetService.applyChangeSet();
  if (response.error) {
    globalErrorMessage$.next(response);
  } else {
    selectedChangeSetPk.value = CHANGE_SET_NONE;
  }
};

const editSessionCancel = async () => {
  let response = await ChangeSetService.cancelEditSession();
  if (response.error) {
    globalErrorMessage$.next(response);
  }
};
const editSessionSave = async () => {
  let response = await ChangeSetService.saveEditSession();
  if (response.error) {
    globalErrorMessage$.next(response);
  }
};
const editSessionStart = async () => {
  let response = await ChangeSetService.startEditSession({
    changeSetPk: selectedChangeSetPk.value,
  });
  if (response.error) {
    globalErrorMessage$.next(response);
  }
};
</script>

<style lang="css" scoped>
.menu-bar {
  background-color: #212121;
}
</style>
