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
          :value="currentRevisionId"
          name="systemSelect"
          :disabled="isRevisionButtonDisabled()"
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
          :disabled="editMode"
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
          size="sm"
          label="cancel"
          class="mx-1"
          icon="null"
          kind="cancel"
          @click.native="cancelChangeSetCreate"
          data-cy="new-change-set-form-cancel-button"
        />
        <SiButton
          size="sm"
          label="create"
          class="mx-1"
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
          size="sm"
          label="cancel"
          class="mx-1"
          icon="null"
          kind="cancel"
          @click.native="cancelChangeSetApply"
          data-cy="new-change-set-form-cancel-button"
        />
        <SiButton
          size="sm"
          label="ok"
          class="mx-1"
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

import { ctxMapState, InstanceStoreContext } from "@/store";
import { PanelEventBus, emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { ChangeSet } from "@/api/sdf/model/changeSet";

import { ApplicationContextStore } from "@/store/modules/applicationContext";
import { SessionStore } from "@/store/modules/session";

import SiError from "@/atoms/SiError.vue";
import SiSelect from "@/atoms/SiSelect.vue";
import SiButton from "@/atoms/SiButton.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";

interface IData {
  selectCurrentChangeSetId: string;
  newChangeSetForm: {
    name: string;
  };
  modalErrorMessage: string;
  editorErrorMessage: string;
}

interface Revision {
  value: string;
  label: string;
}

export default Vue.extend({
  name: "EditorMenuBar",
  props: {
    workspaceId: { type: String },
    applicationId: { type: String },
    applicationContextCtx: {
      type: Object as PropType<InstanceStoreContext<ApplicationContextStore>>,
    },
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
      newChangeSetForm: {
        name: "",
      },
      modalErrorMessage: "",
      editorErrorMessage: "",
    };
  },
  watch: {
    async currentChangeSet(newChangeSet: ChangeSet) {
      if (newChangeSet) {
        this.selectCurrentChangeSetId = newChangeSet.id;
      } else {
        this.selectCurrentChangeSetId = "";
      }
    },
  },
  computed: {
    currentWorkspace(): SessionStore["currentWorkspace"] | undefined {
      return this.$store.state.session.currentWorkspace;
    },
    systemsList(): ApplicationContextStore["systemsList"] | undefined {
      return ctxMapState(this.applicationContextCtx, "systemsList");
    },
    currentSystemId(): SessionStore["currentSystem"] | undefined {
      return this.$store.state.session.currentSystem?.id;
    },
    revisionsList(): Revision[] {
      return [
        {
          value: "",
          label: "latest",
        },
      ];
    },
    currentRevisionId(): string {
      return "latest";
    },
    editMode(): ApplicationContextStore["editMode"] {
      return ctxMapState(this.applicationContextCtx, "editMode");
    },
    currentChangeSet():
      | ApplicationContextStore["currentChangeSet"]
      | undefined {
      return ctxMapState(this.applicationContextCtx, "currentChangeSet");
    },
    // applyButtonIcon(): string {
    //   return !this.currentChangeSet || this.editMode ? "play" : "merge";
    // },
    applyButtonKind(): string {
      return !this.currentChangeSet || this.editMode ? "standard" : "save";
    },
    currentEditSession():
      | ApplicationContextStore["currentEditSession"]
      | undefined {
      return ctxMapState(this.applicationContextCtx, "currentEditSession");
    },
    openChangeSetsList():
      | ApplicationContextStore["openChangeSetsList"]
      | undefined {
      return ctxMapState(this.applicationContextCtx, "openChangeSetsList");
    },
  },
  methods: {
    isRevisionButtonDisabled(): Boolean {
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
    isApplyButtonEnabled(): Boolean {
      if (
        this.selectCurrentChangeSetId &&
        !this.editMode &&
        !this.selectedChangeSetIsAnAction()
      ) {
        return true;
      } else {
        return false;
      }
    },
    isEditButtonEnabled(): Boolean {
      if (
        this.selectCurrentChangeSetId &&
        !this.editMode &&
        !this.selectedChangeSetIsAnAction()
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
      let reply = await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath(
          "createChangeSetAndEditSession",
        ),
        {
          workspaceId: this.currentWorkspace?.id,
          changeSetName: this.newChangeSetForm.name,
        },
      );
      if (reply.error) {
        this.modalErrorMessage = reply.error.message;
      } else {
        this.$emit("update-query-param", {
          changeSetId: reply.changeSet.id,
          editSessionId: reply.editSession.id,
        });
        this.clearChangeSetCreateForm();
        this.$modal.hide("changeSetCreate");
        await this.setEditMode();
      }
    },
    async changeSetSelected() {
      if (this.selectCurrentChangeSetId) {
        if (!this.selectCurrentChangeSetId.includes("action")) {
          let reply = await this.$store.dispatch(
            this.applicationContextCtx.dispatchPath(
              "createEditSessionAndLoadChangeSet",
            ),
            { changeSetId: this.selectCurrentChangeSetId },
          );
          if (reply.error) {
            this.modalErrorMessage = reply.error.message;
          } else {
            await this.$emit("update-query-param", {
              changeSetId: reply.changeSet.id,
              editSessionId: reply.editSession.id,
            });
          }
        } else if (this.selectCurrentChangeSetId.includes("action:new")) {
          await this.showChangeSetCreateModal();
        } else {
          await this.$store.dispatch(
            this.applicationContextCtx.dispatchPath(
              "clearCurrentChangeSetAndCurrentEditSession",
            ),
            null,
            { root: true },
          );
          await this.$emit("remove-query-param", [
            "changeSetId",
            "editSessionId",
          ]);
        }
      } else {
        await this.$store.dispatch(
          this.applicationContextCtx.dispatchPath(
            "clearCurrentChangeSetAndCurrentEditSession",
          ),
          null,
          { root: true },
        );
        await this.$emit("remove-query-param", [
          "changeSetId",
          "editSessionId",
        ]);
      }
    },
    async editModeChangeSetSelected() {
      if (this.selectCurrentChangeSetId) {
        let reply = await this.$store.dispatch(
          this.applicationContextCtx.dispatchPath(
            "createEditSessionAndLoadChangeSet",
          ),
          { changeSetId: this.selectCurrentChangeSetId },
        );
        if (reply.error) {
          this.modalErrorMessage = reply.error.message;
        } else {
          await this.$emit("update-query-param", {
            changeSetId: reply.changeSet.id,
            editSessionId: reply.editSession.id,
          });
          await this.setEditMode();
          this.$modal.hide("changeSetCreate");
        }
      } else {
        console.log("none!");
      }
    },
    async startEditSession() {
      if (this.currentChangeSet) {
        await this.editSessionCreate();
        await this.setEditMode();
      } else {
        await this.showChangeSetCreateModal();
      }
    },
    async saveEditSession() {
      if (this.currentWorkspace && this.currentEditSession) {
        let reply = await this.applicationContextCtx.dispatch(
          "saveEditSession",
          {
            editSessionId: this.currentEditSession.id,
            workspaceId: this.currentWorkspace.id,
          },
        );
        if (reply.error) {
          emitEditorErrorMessage(
            `failed to save edit session: ${reply.error.message}`,
          );
        } else {
          await this.$store.dispatch(
            this.applicationContextCtx.dispatchPath("setEditMode"),
            false,
          );
          this.$emit("update-query-param", { editMode: false });
          this.$emit("remove-query-param", ["editSessionId", "editMode"]);
        }
      }
    },
    async setEditMode() {
      await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath("setEditMode"),
        true,
      );
      this.$emit("update-query-param", { editMode: true });
    },
    async editSessionCreate() {
      let reply = await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath("createEditSession"),
        {
          workspaceId: this.currentWorkspace?.id,
          changeSetId: this.currentChangeSet?.id,
        },
      );
      if (reply.error) {
        this.modalErrorMessage = reply.error.message;
      } else {
        await this.$emit("update-query-param", {
          editSessionId: reply.editSession.id,
        });
        await this.setEditMode();
      }
    },
    async cancelEditSession() {
      let reply = await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath("cancelEditSession"),
        {
          workspaceId: this.currentWorkspace?.id,
          editSessionId: this.currentEditSession?.id,
        },
      );
      if (reply.error) {
        emitEditorErrorMessage(
          `failed to cancel edit session: ${reply.error.message}`,
        );
      } else {
        await this.$store.dispatch(
          this.applicationContextCtx.dispatchPath("setEditMode"),
          false,
        );
        this.$emit("update-query-param", { editMode: false });
        this.$emit("remove-query-param", ["editSessionId", "editMode"]);
      }
    },
    async cancelChangeSetApply() {
      this.clearChangeSetApplyForm();
      this.$modal.hide("changeSetApply");
    },
    async applyChangeSet() {
      await this.showChangeSetApplyModal();
    },
    async changeSetApply() {
      if (this.currentWorkspace && this.currentChangeSet) {
        let reply = await this.applicationContextCtx.dispatch(
          "applyChangeSet",
          {
            changeSetId: this.currentChangeSet.id,
            workspaceId: this.currentWorkspace.id,
          },
        );
        if (reply.error) {
          emitEditorErrorMessage(
            `failed to apply change set: ${reply.error.message}`,
          );
        } else {
          this.$emit("remove-query-param", ["changeSetId"]);
          this.$store.dispatch(
            this.applicationContextCtx.dispatchPath("loadApplicationContext"),
            {
              workspaceId: this.workspaceId,
              applicationId: this.applicationId,
            },
          );
        }
      }
      this.clearChangeSetApplyForm();
      await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath(
          "clearCurrentChangeSetAndCurrentEditSession",
        ),
        null,
        { root: true },
      );
      await this.$emit("remove-query-param", ["changeSetId", "editSessionId"]);
      this.$modal.hide("changeSetApply");
    },
    async showChangeSetCreateModal() {
      await this.$modal.show("changeSetCreate");
    },
    async showChangeSetApplyModal() {
      await this.$modal.show("changeSetApply");
    },
  },
});
</script>

<style lang="css" scoped>
.menu-bar {
  background-color: #212121;
}
</style>
