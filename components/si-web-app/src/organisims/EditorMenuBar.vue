<template>
  <div class="flex w-full">
    <div id="system-selector" class="flex">
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
    </div>

    <div id="changeset-selector" class="flex">
      <div class="flex items-center justify-end pr-1 text-xs text-gray-400">
        changeset:
      </div>
      <div class="flex items-center mr-2">
        <SiSelect
          size="xs"
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
        class="w-16"
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
        label="edit"
        icon="edit"
        size="xs"
        v-else
      />
      <SiButton
        class="w-16 ml-1"
        label="apply"
        :icon="applyButtonIcon"
        :kind="applyButtonKind"
        size="xs"
        v-if="currentChangeSet && !editMode"
        @click.native="applyChangeSet"
      />
    </div>
    <SiModal
      name="changeSetCreate"
      title="Select or create a changeSet"
      class="overflow-visible"
    >
      <div class="flex-row w-full">
        <div class="w-full text-right text-red-400">
          ! a changeSet is required to make edits
        </div>
        <SiError
          testId="change-set-create-error"
          :message="modalErrorMessage"
          @clear="clearModalErrorMessage"
        />
        <div class="items-center w-full">
          <div class="flex items-center w-full">
            <div class="w-1/3 mr-2 text-right">changeSet:</div>
            <div class="w-3/6">
              <SiSelect
                size="sm"
                :options="openChangeSetsList"
                id="selectCurrentChangeSetEdit"
                v-model="selectCurrentChangeSetId"
                name="selectCurrentChangeSetEdit"
                @change.native="editModeChangeSetSelected"
              />
            </div>
          </div>
          <div class="flex items-center w-full mt-4">
            <div class="w-1/3 mr-2 text-right">name:</div>
            <div class="w-3/6">
              <SiTextBox
                class="w-full"
                name="new-change-set-name"
                id="new-change-set-name"
                size="sm"
                placeholder="new change set name"
                v-model="newChangeSetForm.name"
                v-on:keyup.enter.native="changeSetCreate"
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
          @click.native="cancelChangeSetCreate()"
          data-cy="new-change-set-form-cancel-button"
        />
        <SiButton
          size="sm"
          label="create"
          class="m-1"
          icon="save"
          kind="save"
          :disabled="!newChangeSetForm.name"
          @click.native="changeSetCreate"
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
    editMode(): ApplicationContextStore["editMode"] {
      return ctxMapState(this.applicationContextCtx, "editMode");
    },
    currentChangeSet():
      | ApplicationContextStore["currentChangeSet"]
      | undefined {
      return ctxMapState(this.applicationContextCtx, "currentChangeSet");
    },
    applyButtonIcon(): string {
      return !this.currentChangeSet || this.editMode ? "play" : "save";
    },
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
    clearModalErrorMessage() {
      this.modalErrorMessage = "";
    },
    clearChangeSetCreateForm() {
      this.newChangeSetForm.name = "";
      this.modalErrorMessage = "";
    },
    async cancelChangeSetCreate() {
      this.clearChangeSetCreateForm();
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
      }
    },
    async applyChangeSet() {
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
    async showChangeSetCreateModal() {
      await this.$modal.show("changeSetCreate");
    },
    async changeSetSelected() {
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
  },
});
</script>

<style lang="css" scoped>
.menu-bar {
  background-color: #212121;
}
</style>
