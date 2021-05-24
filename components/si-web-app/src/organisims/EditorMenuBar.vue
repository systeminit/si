<template>
  <div class="flex w-full h-6" @keyup.stop @keydown.stop>
    <!-- <div id="system-selector" class="flex">
      <div class="flex items-center justify-end pr-1 text-xs text-gray-400">
        system:
      </div>
      <div class="flex items-center mr-4">
        <SiSelect
          size="xs"
          id="systemSelect"
          :options="systemsList"
          :value="currentSystemId"
          name="systemSelect"
          :disabled="editMode"
        />
      </div>
    </div> -->

    <div id="revision-selector" class="flex">
      <div class="flex items-center justify-end pr-1 text-xs text-gray-400">
        revision:
      </div>
      <div class="flex items-center mr-4">
        <SiSelect
          size="xs"
          id="revisionSelect"
          :styling="changeSetSelectorStyling()"
          :options="revisionsList"
          v-model="currentRevisionId"
          name="systemSelect"
          :disabled="isRevisionButtonDisabled()"
          @change.native="revisionSelected"
        />
      </div>
    </div>

    <div id="changeset-selector" class="flex">
      <div class="flex items-center justify-end pr-1 text-xs text-gray-400">
        changeset:
      </div>
      <div class="flex items-center mr-2">
        <SiSelect
          size="xs"
          :styling="changeSetSelectorStyling()"
          :options="openChangeSetsList"
          id="selectCurrentChangeSet"
          v-model="selectCurrentChangeSetId"
          :disabled="isChangeSetDisabled()"
          @change.native="changeSetSelected"
        />
      </div>
    </div>

    <div id="buttons" class="flex w-auto mr-2">
      <SiButton
        @click.native="cancelEditSession"
        class="w-16 ml-1"
        label="cancel"
        icon="cancel"
        kind="cancel"
        size="xs"
        v-if="editMode"
      />
      <SiButton
        @click.native="saveEditSession"
        class="w-16 ml-1"
        label="save"
        icon="save"
        kind="save"
        size="xs"
        v-if="editMode"
      />
      <SiButton
        class="w-16 ml-1"
        @click.native="startEditSession"
        :label="changeSetInteractionButton()"
        icon="edit"
        size="xs"
        v-if="isEditButtonEnabled()"
      />
      <SiButton
        class="w-16 ml-1"
        label="apply"
        icon="merge"
        :kind="applyButtonKind"
        size="xs"
        v-if="isApplyButtonEnabled()"
        @click.native="applyChangeSet"
      />
    </div>
    <SiModal
      name="changeSetCreate"
      title="Create a changeSet"
      class="w-40 overflow-visible"
    >
      <div class="flex-row">
        <SiError
          testId="change-set-create-error"
          :message="modalErrorMessage"
          @clear="clearModalErrorMessage"
        />
        <div class="flex items-center justify-end -mr-1">
          <div class="text-sm text-right">name:</div>
          <SiTextBox
            class="ml-1"
            name="new-change-set-name"
            id="new-change-set-name"
            size="sm"
            placeholder="new change set name"
            v-model="newChangeSetForm.name"
            v-on:keyup.enter.native="changeSetCreate"
          />
        </div>
      </div>
      <template v-slot:buttons>
        <SiButton
          size="xs"
          label="cancel"
          icon="null"
          kind="cancel"
          @click.native="cancelChangeSetCreate"
          data-cy="new-change-set-form-cancel-button"
        />
        <SiButton
          size="xs"
          label="create"
          class="ml-1"
          icon="null"
          kind="save"
          :disabled="!newChangeSetForm.name"
          @click.native="changeSetCreate"
          data-cy="new-change-set-form-create-button"
        />
      </template>
    </SiModal>
    <SiModal
      name="changeSetApply"
      title="ChangeSet Apply"
      class="overflow-visible"
      width="500px"
    >
      <div class="">
        <div class="flex flex-col">
          <div class="mb-1 ml-1 text-sm text-left">Comment</div>
          <SiTextBox
            class="mb-1 ml-1"
            name="new-change-set-name"
            id="new-change-set-name"
            size="sm"
            :isTextArea="true"
            :isShowType="false"
            placeholder="no comment"
            v-model="newChangeSetForm.name"
          />
        </div>
      </div>
      <template v-slot:buttons>
        <SiButton
          size="xs"
          label="cancel"
          icon="null"
          kind="cancel"
          @click.native="cancelChangeSetApply"
          data-cy="new-change-set-form-cancel-button"
        />
        <SiButton
          size="xs"
          label="ok"
          class="ml-1"
          icon="null"
          kind="save"
          @click.native="changeSetApply"
          data-cy="new-change-set-form-create-button"
        />
      </template>
    </SiModal>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import SiError from "@/atoms/SiError.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import SiButton from "@/atoms/SiButton.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import { Entity } from "@/api/sdf/model/entity";
import { IWorkspace } from "@/api/sdf/model/workspace";
import { switchMap, pluck, tap } from "rxjs/operators";
import { combineLatest, from } from "rxjs";
import { ApplicationContextDal } from "@/api/sdf/dal/applicationContextDal";
import {
  system$,
  editMode$,
  changeSet$,
  editSession$,
  revision$,
} from "@/observables";
import _ from "lodash";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface IData {
  selectCurrentChangeSetId: string;
  currentRevisionId: string;
  newChangeSetForm: {
    name: string;
  };
  modalErrorMessage: string;
  editorErrorMessage: string;
  systemsList: {
    value: string;
    label: string;
  }[];
  openChangeSetsList: {
    value: string;
    label: string;
  }[];
  revisionsList: {
    value: string;
    label: string;
  }[];
}

export default Vue.extend({
  name: "EditorMenuBar",
  props: {
    workspace: { type: Object as PropType<IWorkspace> },
    application: { type: Object as PropType<Entity> },
  },
  components: {
    SiSelect,
    SiButton,
    SiTextBox,
    SiModal,
    SiError,
  },
  data(): IData {
    return {
      selectCurrentChangeSetId: "",
      currentRevisionId: "",
      newChangeSetForm: {
        name: "",
      },
      modalErrorMessage: "",
      editorErrorMessage: "",
      systemsList: [],
      openChangeSetsList: [],
      revisionsList: [],
    };
  },
  subscriptions(this: any): Record<string, any> {
    let currentApplication$ = this.$watchAsObservable("application", {
      immediate: true,
    }).pipe(pluck("newValue"));
    let currentWorkspace$ = this.$watchAsObservable("workspace", {
      immediate: true,
    }).pipe(pluck("newValue"));
    return {
      applicationContext: combineLatest(
        currentApplication$,
        currentWorkspace$,
        changeSet$,
        editSession$,
      ).pipe(
        switchMap(([currentApplication, currentWorkspace]) => {
          if (currentApplication && currentWorkspace) {
            return from(
              ApplicationContextDal.getApplicationContext({
                // @ts-ignore
                applicationId: currentApplication.id,
                // @ts-ignore
                workspaceId: currentWorkspace.id,
              }),
            );
          } else {
            return from([
              {
                error: {
                  code: 42,
                  message:
                    "cannot load application context without workspace or application id",
                },
              },
            ]);
          }
        }),
        tap(reply => {
          if (reply.error) {
            if (reply.error.code != 42) {
              emitEditorErrorMessage(reply.error.message);
            }
          } else {
            this.systemsList = reply.systemsList;
            this.openChangeSetsList = _.concat(reply.openChangeSetsList, [
              { label: "- none -", value: "" },
              { label: ": new :", value: "action:new" },
            ]);
            this.revisionsList = _.concat(reply.revisionsList, [
              { value: "", label: "- latest -" },
            ]);
          }
        }),
      ),
      currentSystemId: system$.pipe(switchMap(system => from([system?.id]))),
      currentChangeSet: changeSet$.pipe(
        tap(changeSet => {
          if (changeSet && !this.currentRevisionId) {
            this.selectCurrentChangeSetId = changeSet.id;
          } else {
            this.selectCurrentChangeSetId = "";
          }
        }),
      ),
      currentEditSession: editSession$,
      editMode: editMode$,
    };
  },
  computed: {
    // applyButtonIcon(): string {
    //   return !this.currentChangeSet || this.editMode ? "play" : "merge";
    // },
    applyButtonKind(): string {
      // @ts-ignore
      return !this.currentChangeSet || this.editMode ? "standard" : "save";
    },
  },
  methods: {
    isRevisionButtonDisabled(): Boolean {
      // @ts-ignore
      if (this.editMode) {
        return true;
      } else if (
        this.selectCurrentChangeSetId &&
        !this.selectedChangeSetIsAnAction()
      ) {
        return true;
      } else {
        return false;
      }
    },
    isChangeSetDisabled(): Boolean {
      if (
        // @ts-ignore
        !this.editMode &&
        this.currentRevisionId
      ) {
        return true;
      } else {
        return false;
      }
    },
    isApplyButtonEnabled(): Boolean {
      if (
        this.selectCurrentChangeSetId &&
        // @ts-ignore
        !this.editMode &&
        !this.selectedChangeSetIsAnAction() &&
        !this.currentRevisionId
      ) {
        return true;
      } else {
        return false;
      }
    },
    isEditButtonEnabled(): Boolean {
      if (
        this.selectCurrentChangeSetId &&
        // @ts-ignore
        !this.editMode &&
        !this.selectedChangeSetIsAnAction() &&
        !this.currentRevisionId
      ) {
        return true;
      } else {
        return false;
      }
    },
    changeSetInteractionButton(): string {
      // if (this.currentChangeSet == null) {
      //   return "new"
      // } else {
      //   return "edit"
      // }
      return "edit";
    },
    changeSetSelectorStyling(): Record<string, any> {
      let classes: Record<string, any> = {};
      classes["bg-selector1"] = true;
      classes["text-gray-400"] = true;
      classes["border-gray-700"] = true;
      return classes;
    },
    selectedChangeSetIsAnAction(): Boolean {
      if (
        this.selectCurrentChangeSetId &&
        this.selectCurrentChangeSetId.includes("action")
      ) {
        return true;
      } else {
        return false;
      }
    },
    clearModalErrorMessage() {
      this.modalErrorMessage = "";
    },
    clearChangeSetCreateForm() {
      this.newChangeSetForm.name = "";
      this.modalErrorMessage = "";
    },
    clearChangeSetApplyForm() {
      this.newChangeSetForm.name = "";
      this.modalErrorMessage = "";
    },
    clearSelectCurrentChangeSetId() {
      this.selectCurrentChangeSetId = "";
    },
    async cancelChangeSetCreate() {
      this.clearChangeSetCreateForm();
      this.clearSelectCurrentChangeSetId();
      this.$modal.hide("changeSetCreate");
    },
    async changeSetCreate() {
      let reply = await ApplicationContextDal.createChangeSetAndEditSession({
        workspaceId: this.workspace?.id,
        changeSetName: this.newChangeSetForm.name,
      });
      if (reply.error) {
        this.modalErrorMessage = reply.error.message;
      } else {
        changeSet$.next(reply.changeSet);
        editSession$.next(reply.editSession);
        this.clearChangeSetCreateForm();
        this.$modal.hide("changeSetCreate");
        await this.setEditMode();
      }
    },
    async revisionSelected() {
      if (this.currentRevisionId) {
        let reply = await ApplicationContextDal.getChangeSet({
          changeSetId: this.currentRevisionId,
        });
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        } else {
          revision$.next(reply.changeSet);
          changeSet$.next(reply.changeSet);
        }
      } else {
        revision$.next(null);
        changeSet$.next(null);
      }
    },
    async changeSetSelected() {
      if (this.selectCurrentChangeSetId) {
        if (!this.selectCurrentChangeSetId.includes("action")) {
          let reply = await ApplicationContextDal.createEditSessionAndGetChangeSet(
            { changeSetId: this.selectCurrentChangeSetId },
          );
          if (reply.error) {
            this.modalErrorMessage = reply.error.message;
          } else {
            changeSet$.next(reply.changeSet);
            editSession$.next(reply.editSession);
          }
        } else if (this.selectCurrentChangeSetId.includes("action:new")) {
          this.showChangeSetCreateModal();
        } else {
          changeSet$.next(null);
          editSession$.next(null);
          revision$.next(null);
          editMode$.next(false);
        }
      } else {
        changeSet$.next(null);
        editSession$.next(null);
        revision$.next(null);
        editMode$.next(false);
      }
    },
    async startEditSession() {
      // @ts-ignore
      if (this.currentChangeSet) {
        await this.editSessionCreate();
        await this.setEditMode();
      } else {
        this.showChangeSetCreateModal();
      }
    },
    async saveEditSession() {
      // @ts-ignore
      if (this.workspace && this.currentEditSession) {
        let reply = await ApplicationContextDal.saveEditSession({
          // @ts-ignore
          editSessionId: this.currentEditSession.id,
          workspaceId: this.workspace?.id,
        });
        if (reply.error) {
          emitEditorErrorMessage(
            `failed to save edit session: ${reply.error.message}`,
          );
        } else {
          editSession$.next(null);
          editMode$.next(false);
        }
      }
    },
    async setEditMode() {
      editMode$.next(true);
    },
    async editSessionCreate() {
      let reply = await ApplicationContextDal.createEditSession({
        workspaceId: this.workspace?.id,
        // @ts-ignore
        changeSetId: this.currentChangeSet?.id,
      });
      if (reply.error) {
        this.modalErrorMessage = reply.error.message;
      } else {
        editSession$.next(reply.editSession);
        await this.setEditMode();
      }
    },
    async cancelEditSession() {
      let reply = await ApplicationContextDal.cancelEditSession({
        workspaceId: this.workspace?.id,
        // @ts-ignore
        editSessionId: this.currentEditSession?.id,
      });
      if (reply.error) {
        emitEditorErrorMessage(
          `failed to cancel edit session: ${reply.error.message}`,
        );
      } else {
        editSession$.next(null);
        editMode$.next(false);
      }
    },
    async cancelChangeSetApply() {
      this.clearChangeSetApplyForm();
      this.$modal.hide("changeSetApply");
    },
    async applyChangeSet() {
      this.showChangeSetApplyModal();
    },
    async changeSetApply() {
      // @ts-ignore
      if (this.workspace && this.currentChangeSet) {
        let reply = await ApplicationContextDal.applyChangeSet({
          // @ts-ignore
          changeSetId: this.currentChangeSet.id,
          workspaceId: this.workspace?.id,
        });
        if (reply.error) {
          emitEditorErrorMessage(
            `failed to apply change set: ${reply.error.message}`,
          );
        } else {
          changeSet$.next(null);
          editSession$.next(null);
          editMode$.next(false);
        }
      }
      this.clearChangeSetApplyForm();
      this.$modal.hide("changeSetApply");
    },
    showChangeSetCreateModal() {
      this.$modal.show("changeSetCreate");
    },
    showChangeSetApplyModal() {
      this.$modal.show("changeSetApply");
    },
  },
});
</script>

<style lang="css" scoped>
.menu-bar {
  background-color: #212121;
}
</style>
