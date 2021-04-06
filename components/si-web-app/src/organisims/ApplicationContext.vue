<template>
  <div class="flex flex-col w-full pb-2 header-background">
    <SiError
      testId="editor-error"
      :message="editorErrorMessage"
      @clear="clearEditorErrorMessage"
    />
    <div class="flex justify-between" :class="titleBarClasses()">
      <div class="flex items-center">
        <button
          @click="toggleDetails"
          class="focus:outline-none"
          data-cy="application-details-toggle"
        >
          <ChevronDownIcon
            v-if="showDetails"
            size="1.2x"
            class="inline-flex text-gray-300"
          />
          <ChevronRightIcon
            size="1.2x"
            v-else
            class="inline-flex text-gray-300"
          />
        </button>
        <div
          class="inline-flex font-light text-gray-300 font-small "
          data-cy="application-details-application-name"
        >
          {{ applicationName }}
        </div>
      </div>

      <div class="flex items-center mr-2">
        <EditorMenuBar
          :applicationContextCtx="applicationContextCtx"
          :workspaceId="workspaceId"
          :applicationId="applicationId"
          v-show="!showDetails"
        />
      </div>
    </div>
    <div
      class="flex w-full pb-2 details-panel-background"
      data-cy="application-details-extended"
      v-show="showDetails"
    >
      <div class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 details-panel card-section">
        <div class="details-panel-title">Activity</div>
      </div>
      <div class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 details-panel card-section">
        <div class="details-panel-title">Services</div>
      </div>

      <div class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 details-panel card-section">
        <div class="details-panel-title">Resources</div>
      </div>
      <div class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 details-panel card-section">
        <div class="flex flex-col">
          <div class="flex flex-row align-middle">
            <div class="self-center">
              <div class="details-panel-title">Changeset</div>
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
      </div>
    </div>

    <div class="flex justify-end mt-1 mr-2" v-show="showDetails">
      <div class="flex items-center justify-end">
        <EditorMenuBar
          :applicationContextCtx="applicationContextCtx"
          :workspaceId="workspaceId"
          :applicationId="applicationId"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { ctxMapState, InstanceStoreContext } from "@/store";
import { ApplicationContextStore } from "@/store/modules/applicationContext";
import { SessionStore } from "@/store/modules/session";
import { IEditorContext } from "@/store/modules/editor";
import Vue, { PropType } from "vue";

import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";
import SiError from "@/atoms/SiError.vue";
import { ChangeSet } from "@/api/sdf/model/changeSet";
import { SDFError } from "@/api/sdf";
import { PanelEventBus, emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import EditorMenuBar from "@/organisims/EditorMenuBar.vue";

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
    EditorMenuBar,
    ChevronRightIcon,
    ChevronDownIcon,
    SiError,
  },
  data(): IData {
    return {
      showDetails: false,
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
    applyButtonKind(): string {
      return !this.currentChangeSet || this.editMode ? "standard" : "save";
    },
    applyButtonIcon(): string {
      return !this.currentChangeSet || this.editMode ? "play" : "save";
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
    titleBarClasses(): Record<string, any> {
      let classes: Record<string, any> = {};
      classes["title-background"] = this.showDetails;
      classes["mt-2"] = !this.showDetails;
      return classes;
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
    async setEditorErrorMessage(error: string) {
      this.editorErrorMessage = error;
    },
    clearEditorErrorMessage() {
      this.editorErrorMessage = "";
    },
    clearModalErrorMessage() {
      this.modalErrorMessage = "";
    },
  },
  async created() {
    // @ts-ignore
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

<style scoped>
.details-panel {
  border: solid;
  border-width: 1px;
  border-color: #464753;
}

.details-panel-title {
  @apply font-normal text-xs;
}

.details-panel-background {
  background-color: #171717;
}

.header-background {
  background-color: #171717;
}

.title-background {
  background-color: #292929;
}
</style>
