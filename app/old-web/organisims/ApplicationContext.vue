<template>
  <div class="flex flex-col w-full pb-2 header-background">
    <SiError
      testId="editor-error"
      :message="editorErrorMessage"
      @clear="clearEditorErrorMessage"
    />
    <div class="flex justify-between mt-2">
      <div class="flex items-center">
        <button
          @click="toggleDetails"
          class="focus:outline-none"
          data-cy="application-details-toggle"
        >
          <ChevronDownIcon
            v-if="showDetails"
            size="1.1x"
            class="text-gray-300 "
          />
          <ChevronRightIcon size="1.1x" v-else class="text-gray-300 " />
        </button>
      </div>

      <MenuSummary
        :applicationId="application.id"
        v-if="!showDetails && application"
      />

      <div class="flex mr-2" v-if="!showDetails">
        <EditorMenuBar
          :workspace="currentWorkspace"
          :application="application"
        />
      </div>
    </div>
    <div
      class="flex w-full h-full pb-2 details-panel-background"
      data-cy="application-details-extended"
      v-if="showDetails"
    >
      <div class="w-1/5 h-full py-2 mx-2 ">
        <ActivitySummary :applicationId="application.id" v-if="application" />
      </div>

      <div class="w-1/5 h-full py-2 mx-2 ">
        <ServicesSummary :applicationId="application.id" v-if="application" />
      </div>

      <div class="w-1/5 h-full py-2 mx-2 ">
        <ComputingResourceSummary
          :applicationId="application.id"
          v-if="application"
        />
      </div>

      <div class="w-1/5 h-full py-2 mx-2 ">
        <ProviderSummary :applicationId="application.id" v-if="application" />
      </div>

      <div class="w-1/5 h-full py-2 mx-2 ">
        <ChangesSummary :applicationId="application.id" v-if="application" />
      </div>
    </div>

    <div class="flex justify-end mt-1 mr-2" v-if="showDetails">
      <div class="flex items-center justify-end">
        <EditorMenuBar
          :workspace="currentWorkspace"
          :application="application"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";
import SiError from "@/atoms/SiError.vue";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import EditorMenuBar from "@/organisims/EditorMenuBar.vue";

import ActivitySummary from "@/molecules/ActivitySummary.vue";
import ServicesSummary from "@/molecules/ServicesSummary.vue";
import ComputingResourceSummary from "@/molecules/ComputingResourceSummary.vue";
import ProviderSummary from "@/molecules/ProviderSummary.vue";
import ChangesSummary from "@/molecules/ChangesSummary.vue";
import MenuSummary from "@/molecules/MenuSummary.vue";

import { Entity } from "@/api/sdf/model/entity";
import { workspace$, editMode$, changeSet$, editSession$ } from "@/observables";

interface IData {
  showDetails: boolean;
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
    application: { type: Object as PropType<Entity> },
  },
  components: {
    MenuSummary,
    EditorMenuBar,
    ChevronRightIcon,
    ChevronDownIcon,
    SiError,
    ActivitySummary,
    ServicesSummary,
    ComputingResourceSummary,
    ChangesSummary,
    ProviderSummary,
    // UploadIcon,
    // SiButton,
  },
  data(): IData {
    return {
      showDetails: false,
      newChangeSetForm: {
        name: "",
      },
      modalErrorMessage: "",
      editorErrorMessage: "",
    };
  },
  subscriptions(): Record<string, any> {
    return {
      currentWorkspace: workspace$,
      editMode: editMode$,
      currentChangeSet: changeSet$,
      currentEditSession: editSession$,
    };
  },
  computed: {
    applicationName(): string | undefined {
      return this.application.name;
    },
    applyButtonKind(this: any): string {
      return !this.currentChangeSet || this.editMode ? "standard" : "save";
    },
    applyButtonIcon(this: any): string {
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
      // classes["title-background"] = this.showDetails;
      return classes;
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
    PanelEventBus.$on("editor-error-message", this.setEditorErrorMessage);
  },
  async beforeDestroy() {
    PanelEventBus.$off("editor-error-message", this.setEditorErrorMessage);
  },
});
</script>

<style scoped>
.details-panel {
  border: solid;
  border-width: 1px;
  border-color: #464753;
  background-color: #101010;
}

.details-panel-title {
  /* @apply font-normal text-xs; */
  font-weight: 400;
  font-size: 0.75rem;
  line-height: 1rem;
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
