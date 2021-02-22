<template>
  <div class="flex flex-col w-full pb-3">
    <SiError testId="editor-error" :message="editorErrorMessage" />
    <div class="flex mt-3">
      <div class="items-center w-1/2">
        <button
          @click="toggleDetails"
          class="focus:outline-none"
          data-cy="application-details-toggle"
        >
          <ChevronDownIcon
            v-if="showDetails"
            class="inline-flex text-gray-300"
          />
          <ChevronRightIcon v-else class="inline-flex text-gray-300" />
        </button>
        <div
          class="inline-flex font-normal text-gray-300"
          data-cy="application-details-application-name"
        >
          applications/{{ applicationName }}
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
            id="systemSelect"
            :options="systemsList"
            :value="currentSystemId"
            name="systemSelect"
            :disabled="editMode"
          />
        </div>
        <div
          class="inline-flex justify-end mr-2 font-normal text-gray-400 w-14"
        >
          <SiButton
            @click.native="cancelEditSession"
            label="cancel"
            icon="cancel"
            kind="cancel"
            size="xs"
            v-if="editMode"
          />
        </div>
        <div
          class="inline-flex justify-end w-16 mr-2 font-normal text-gray-400"
        >
          <SiButton
            @click.native="finishEditSession"
            class="w-16"
            label="done"
            icon="save"
            kind="save"
            size="xs"
            v-if="editMode"
          />

          <SiButton
            class="w-16"
            @click.native="startEditSession"
            label="edit"
            icon="edit"
            size="xs"
            v-else
          />
        </div>
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
    <transition
      enter-active-class="transition-all delay-75 ease-out"
      leave-active-class="transition-all delay-75 ease-in"
      enter-class="opacity-0 scale-0"
      enter-to-class="opacity-100 scale-100"
      leave-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-75"
    >
      <div
        class="flex w-full"
        data-cy="application-details-extended"
        v-show="showDetails"
      >
        <div
          class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 border border-solid card-section"
        >
          Activity Visualiation
        </div>
        <div
          class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 border border-solid card-section"
        >
          Services Visualization
        </div>

        <div
          class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 border border-solid card-section"
        >
          Resources Visualization
        </div>
        <div
          class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 border border-solid card-section"
        >
          <div class="flex flex-col">
            <div class="flex flex-row align-middle">
              <div class="self-center text-sm font-bold text-gray-400">
                changeset:
              </div>
              <div class="flex ml-2">
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
            <div class="flex flex-row text-xs text-gray-400 align-middle">
              <div>participants:</div>
              <div class="ml-2">
                <!-- <template v-if="changeSetParticipantCount == 0"> -->
                <template v-if="true"> 0 (fake) </template>
                <template v-else>
                  <span class="text-gold">
                    {{ changeSetParticipantCount }}
                  </span>
                </template>
              </div>
            </div>
          </div>
          <div class="flex justify-end w-full pt-2 pr-1">
            <SiButton
              label="execute"
              icon="play"
              size="xs"
              :disabled="!currentChangeSet"
            />
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script lang="ts">
import { ctxMapState, InstanceStoreContext } from "@/store";
import { ApplicationContextStore } from "@/store/modules/applicationContext";
import { SessionStore } from "@/store/modules/session";
import { IEditorContext } from "@/store/modules/editor";
import Vue, { PropType } from "vue";

import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";
import SiSelect from "@/atoms/SiSelect.vue";
import SiButton from "@/atoms/SiButton.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiTextBox from "@/atoms/SiTextBox.vue";
import SiError from "@/atoms/SiError.vue";
import { ChangeSet } from "@/api/sdf/model/changeSet";
import { SDFError } from "@/api/sdf";
import { PanelEventBus } from "@/atoms/PanelEventBus";

interface IData {
  showDetails: boolean;
  selectCurrentChangeSetId: string;
  newChangeSetForm: {
    name: string;
  };
  modalErrorMessage: string;
  editorErrorMessage: string;
}

export default Vue.extend({
  name: "ApplicationContext",
  props: {
    workspaceId: { type: String },
    applicationId: { type: String },
    applicationContextCtx: {
      type: Object as PropType<InstanceStoreContext<ApplicationContextStore>>,
    },
  },
  components: {
    ChevronRightIcon,
    ChevronDownIcon,
    SiSelect,
    SiButton,
    SiModal,
    SiTextBox,
    SiError,
  },
  data(): IData {
    return {
      showDetails: true,
      selectCurrentChangeSetId: "",
      newChangeSetForm: {
        name: "",
      },
      modalErrorMessage: "",
      editorErrorMessage: "",
    };
  },
  computed: {
    currentWorkspace(): SessionStore["currentWorkspace"] | undefined {
      return this.$store.state.session.currentWorkspace;
    },
    applicationName(): ApplicationContextStore["applicationName"] | undefined {
      return ctxMapState(this.applicationContextCtx, "applicationName");
    },
    systemsList(): ApplicationContextStore["systemsList"] | undefined {
      return ctxMapState(this.applicationContextCtx, "systemsList");
    },
    editMode(): ApplicationContextStore["editMode"] {
      return ctxMapState(this.applicationContextCtx, "editMode");
    },
    currentSystemId(): SessionStore["currentSystem"] | undefined {
      return this.$store.state.session.currentSystem?.id;
    },
    openChangeSetsList():
      | ApplicationContextStore["openChangeSetsList"]
      | undefined {
      return ctxMapState(this.applicationContextCtx, "openChangeSetsList");
    },
    currentChangeSet():
      | ApplicationContextStore["currentChangeSet"]
      | undefined {
      return ctxMapState(this.applicationContextCtx, "currentChangeSet");
    },
    currentEditSession():
      | ApplicationContextStore["currentEditSession"]
      | undefined {
      return ctxMapState(this.applicationContextCtx, "currentEditSession");
    },
  },
  methods: {
    toggleDetails() {
      if (this.showDetails) {
        this.showDetails = false;
      } else {
        this.showDetails = true;
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
    async cancelChangeSetCreate() {
      this.newChangeSetForm.name = "";
      this.modalErrorMessage = "";
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
        await this.$emit("update-query-param", {
          changeSetId: reply.changeSet.id,
          editSessionId: reply.editSession.id,
        });
        await this.$modal.hide("changeSetCreate");
        await this.setEditMode();
      }
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
    async showChangeSetCreateModal() {
      await this.$modal.show("changeSetCreate");
    },
    async setEditMode() {
      await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath("setEditMode"),
        true,
      );
      this.$emit("update-query-param", { editMode: true });
    },
    async startEditSession() {
      if (this.currentChangeSet) {
        await this.editSessionCreate();
        await this.setEditMode();
      } else {
        await this.showChangeSetCreateModal();
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
      await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath("setEditMode"),
        false,
      );
      this.$emit("update-query-param", { editMode: false });
      this.$emit("remove-query-param", ["editSessionId", "editMode"]);
    },
    async finishEditSession() {
      await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath("setEditMode"),
        false,
      );
      await this.$store.dispatch(
        this.applicationContextCtx.dispatchPath("finishEditSession"),
      );
      this.$emit("update-query-param", { editMode: false });
      this.$emit("remove-query-param", ["editSessionId", "editMode"]);
    },
    async setEditorErrorMessage(error: string) {
      this.editorErrorMessage = error;
    },
  },
  async created() {
    let context: IEditorContext = {
      applicationId: this.applicationId,
    };
    this.$store.dispatch("editor/setContext", context);
    this.$store.dispatch(
      this.applicationContextCtx.dispatchPath("loadApplicationContext"),
      {
        workspaceId: this.workspaceId,
        applicationId: this.applicationId,
      },
    );
    PanelEventBus.$on("editor-error-message", this.setEditorErrorMessage);
  },
  async beforeDestroy() {
    PanelEventBus.$off("editor-error-message", this.setEditorErrorMessage);
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
});
</script>
