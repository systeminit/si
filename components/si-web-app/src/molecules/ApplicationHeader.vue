<template>
  <div class="flex">
    <div class="items-center w-1/2">
      <button
        @click="toggleDetails"
        class="focus:outline-none"
        data-cy="application-details-toggle"
      >
        <ChevronDownIcon v-if="showDetails" class="inline-flex text-gray-300" />
        <ChevronRightIcon v-else class="inline-flex text-gray-300" />
      </button>
      <div
        class="inline-flex font-normal text-gray-300"
        data-cy="application-details-application-name"
      >
        applications/{{ application.name }}
      </div>
    </div>

    <div class="flex items-center justify-end w-1/2 mr-2">
      <div
        class="flex items-center justify-end w-1/4 pr-1 text-xs text-gray-400"
      >
        system:
      </div>
      <div class="flex items-center mr-5">
        <SiSelect
          size="xs"
          class="mr-4"
          :options="systemList"
          v-model="currentSystem"
          name="systemSelect"
          :disabled="isEditMode"
        />
      </div>
      <div class="inline-flex justify-end mr-2 font-normal text-gray-400 w-14">
        <SiButton
          @click.native="cancelEditSession"
          data-cy="application-details-mode-toggle"
          label="cancel"
          icon="cancel"
          kind="cancel"
          size="xs"
          v-if="isEditMode"
        />
      </div>
      <div
        class="inline-flex justify-end w-16 mr-2 font-normal text-gray-400"
        data-cy="application-details-current-mode"
      >
        <SiButton
          @click.native="finishEditSession"
          data-cy="application-details-mode-toggle"
          class="w-16"
          label="done"
          icon="save"
          kind="save"
          size="xs"
          v-if="isEditMode"
        />

        <SiButton
          class="w-16"
          @click.native="startEditSession"
          data-cy="application-details-mode-toggle"
          label="edit"
          icon="edit"
          size="xs"
          v-else
        />
      </div>
      <SiModal
        name="changeSetCreate"
        title="Select or create a changeSet"
        :show.sync="showChangeSetCreateModal"
        class="overflow-visible"
      >
        <div class="flex-row w-full">
          <div class="w-full text-right text-red-400">
            ! a changeSet is required to make edits
          </div>
          <div class="items-center w-full">
            <div class="flex items-center w-full">
              <div class="w-1/3 mr-2 text-right">changeSet:</div>
              <div class="w-3/6">
                <SiSelect
                  size="sm"
                  :options="changeSetOpenList"
                  v-model="currentChangeSet"
                  name="popup"
                  @change.native="modalChangeSetCreateSelected"
                />
              </div>
            </div>
            <div class="flex items-center w-full mt-4">
              <div class="w-1/3 mr-2 text-right">name:</div>
              <div class="w-3/6">
                <SiTextBox
                  class="w-full"
                  name="new-change-set-name"
                  size="sm"
                  placeholder="new change set name"
                  dataCy="new-change-set-form-name"
                  v-model="newChangeSetName"
                  v-on:keyup.enter.native="createChangeSetOnEnter()"
                />
              </div>
            </div>
          </div>
        </div>
        <template v-slot:buttons>
          <SiButton
            size="sm"
            label="cancel"
            class="m-1"
            icon="cancel"
            kind="cancel"
            @click.native="closeChangeSetCreate()"
            data-cy="new-change-set-form-cancel-button"
          />
          <SiButton
            size="sm"
            label="create"
            class="m-1"
            icon="save"
            kind="save"
            :disabled="!newChangeSetName"
            @click.native="createChangeSet()"
            data-cy="new-change-set-form-create-button"
          />
        </template>
      </SiModal>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";
import SiButton from "@/atoms/SiButton.vue";

export default Vue.extend({
  name: "ApplicationHeader",
  components: {
    ChevronRightIcon,
    ChevronDownIcon,
  },
});
</script>
