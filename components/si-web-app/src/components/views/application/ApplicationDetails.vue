<template>
  <div id="application-details" class="flex flex-col">
    <div id="application-summary" class="flex flex-col w-full pb-3">
      <StatusBar class />

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
            applications/{{ application.name }}
          </div>
        </div>
        <div class="flex items-center justify-end w-1/2 mr-2">
          <div
            class="flex items-center justify-end w-1/4 pr-1 text-xs text-gray-400"
          >
            system:
          </div>
          <div class="flex items-center">
            <SiSelect
              size="xs"
              class="mr-4"
              :options="systemList"
              v-model="currentSystem"
              name="systemSelect"
            />
          </div>
          <div
            class="inline-flex justify-end w-8 mr-2 font-normal text-gray-400"
            data-cy="application-details-current-mode"
          >
            <template v-if="isEditMode">edit</template>
            <template v-else>view</template>
          </div>
          <button
            @click="modeSwitch"
            class="focus:outline-none"
            data-cy="application-details-mode-toggle"
          >
            <ToggleRightIcon
              class="inline-flex text-gray-300"
              v-if="isEditMode"
            />
            <ToggleLeftIcon class="inline-flex text-gray-300" v-else />
          </button>
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
                      v-model="newChangeSetName"
                      v-on:keyup.enter.native="createChangeSetOnEnter()"
                    />
                  </div>
                </div>
              </div>
            </div>
            <template v-slot:buttons>
              <Button2
                size="sm"
                label="cancel"
                class="m-1"
                icon="cancel"
                kind="cancel"
                @click.native="closeChangeSetCreate()"
              />
              <Button2
                size="sm"
                label="create"
                class="m-1"
                icon="save"
                kind="save"
                :disabled="!newChangeSetName"
                @click.native="createChangeSet()"
              />
            </template>
          </SiModal>
        </div>
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
            <ActivityVisualization />
          </div>
          <div
            class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 border border-solid card-section"
          >
            <ServicesVisualization :applicationId="applicationId" />
          </div>
          <div
            class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 border border-solid card-section"
          >
            <ResourcesVisualization />
          </div>
          <div
            class="w-1/4 pt-2 pb-2 pl-2 mx-3 mt-2 border border-solid card-section"
          >
            <div class="flex">
              <div class="flex-row w-full text-sm font-bold text-gray-400">
                change: {{ changeSetOpenCount }}
              </div>
            </div>
            <div class="flex">
              <div
                class="flex items-center justify-end w-2/6 pr-1 text-xs text-gray-400"
              >
                changeSet:
              </div>
              <div class="flex items-center w-full pr-1">
                <SiSelect
                  size="xs"
                  class="mr-4"
                  :options="changeSetOpenList"
                  v-model="currentChangeSet"
                  name="box"
                />
              </div>
            </div>
            <div class="flex justify-end w-full pt-2 pr-1">
              <Button2
                label="execute"
                icon="play"
                size="xs"
                :disabled="!currentChangeSet"
                @click.native="changeSetExecute"
              />
            </div>
          </div>
        </div>
      </transition>
    </div>
    <div id="editor" class="flex w-full h-full overflow-hidden">
      <Editor />
    </div>
    <div id="eventBar" class="w-full">
      <EventBar />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapActions, Store } from "vuex";
import { registry } from "si-registry";

import Editor from "@/components/views/editor/Editor.vue";
import StatusBar from "@/components/common/StatusBar.vue";
import EventBar from "@/components/common/EventBar.vue";
import ServicesVisualization from "@/components/visualization/ServicesVisualization.vue";
import ActivityVisualization from "@/components/visualization/ActivityVisualization.vue";
import ResourcesVisualization from "@/components/visualization/ResourcesVisualization.vue";
import Button2 from "@/components/ui/Button2.vue";
import SiModal from "@/components/ui/SiModal.vue";
import SiSelect from "@/components/ui/SiSelect.vue";
import SiTextBox from "@/components/ui/SiTextBox.vue";
import { DropdownProps } from "@/components/ui/Dropdown2.vue";
import { RootStore } from "@/store";
import { ChangeSet, ApplicationEntity, System } from "@/graphql-types";
import _ from "lodash";

import {
  ChevronRightIcon,
  ChevronDownIcon,
  ToggleLeftIcon,
  ToggleRightIcon,
  PlayIcon,
  EditIcon,
  AlertCircleIcon,
  TrashIcon,
  SaveIcon,
} from "vue-feather-icons";

interface Data {
  showDetails: boolean;
  selected: string;
  showChangeSetCreateModal: boolean;
  newChangeSetName: string;
}

export default Vue.extend({
  name: "ApplicationDetails",
  components: {
    Editor,
    StatusBar,
    ChevronRightIcon,
    ChevronDownIcon,
    ToggleLeftIcon,
    ToggleRightIcon,
    ActivityVisualization,
    ServicesVisualization,
    ResourcesVisualization,
    Button2,
    SiModal,
    SiSelect,
    SiTextBox,
    EventBar,
    //PlayIcon,
    //EditIcon,
    //AlertCircleIcon,
    //TrashIcon,
    //SaveIcon,
  },
  props: {
    organizationId: {
      type: String,
    },
    workspaceId: {
      type: String,
    },
    applicationId: {
      type: String,
    },
  },
  data(): Data {
    return {
      showDetails: true,
      selected: "",
      showChangeSetCreateModal: false,
      newChangeSetName: "",
    };
  },
  methods: {
    changeSetExecute() {
      this.$store.dispatch("changeSet/execute");
      this.$store.commit("editor/setMode", "view");
    },
    toggleDetails() {
      this.showDetails = !this.showDetails;
    },
    closeChangeSetCreate() {
      this.showChangeSetCreateModal = false;
    },
    modalChangeSetCreateSelected() {
      this.modeSwitch();
      this.showChangeSetCreateModal = false;
    },
    modeSwitch() {
      if (this.mode == "view" && !this.currentChangeSet) {
        this.showChangeSetCreateModal = true;
      } else {
        this.$store.dispatch("editor/modeSwitch");
      }
    },
    async createChangeSetOnEnter() {
      if (this.newChangeSetName) {
        await this.createChangeSet();
      }
    },
    async createChangeSet() {
      const createdByUserId: string = this.$store.getters["user/userId"];
      const workspaceId: string = this.$store.getters[
        "user/currentWorkspaceId"
      ];
      await this.$store.dispatch("changeSet/create", {
        name: this.newChangeSetName,
        displayName: this.newChangeSetName,
        createdByUserId,
        workspaceId,
      });
      this.showChangeSetCreateModal = false;
      this.newChangeSetName = "";
      this.$store.dispatch("editor/modeSwitch");
    },
  },
  computed: {
    ...mapState({
      mode: (state: any): RootStore["editor"]["mode"] => state.editor.mode,
    }),
    currentChangeSet: {
      get(): RootStore["changeSet"]["current"] {
        return this.$store.state.changeSet.current;
      },
      set(changeSetId: null | string) {
        if (this.mode == "edit" && changeSetId == null) {
          this.$store.commit("editor/setMode", "view");
        }
        this.$store.dispatch("changeSet/setCurrentById", changeSetId);
      },
    },
    systems(): System[] {
      return this.$store.getters["system/forApplicationId"](this.applicationId);
    },
    changeSetOpenCount(): number {
      return this.$store.getters["changeSet/count"]({
        forId: this.applicationId,
        status: "OPEN",
      });
    },
    changeSetOpenList(): DropdownProps["options"] {
      let result: DropdownProps["options"] = _.map(
        this.$store.getters["changeSet/open"],
        (changeSet: ChangeSet) => {
          return {
            value: changeSet.id || "no id",
            label: changeSet.name || "no name",
          };
        },
      );
      result.unshift({ label: "none", value: null });
      return result;
    },
    currentSystem: {
      get(): System {
        if (
          !this.$store.state.system.current &&
          this.$store.state.system.systems &&
          this.$store.state.system.systems[0]
        ) {
          this.$store.commit(
            "system/current",
            this.$store.state.system.systems[0],
          );
        }
        return this.$store.state.system.current;
      },
      set(systemId: null | string) {
        this.$store.dispatch("system/setCurrentById", systemId);
      },
    },
    systemList(): DropdownProps["options"] {
      const systemList = _.map(this.systems, system => {
        return { value: system.id || "no id", label: system.name || "no name" };
      });

      if (systemList) {
        return systemList;
      } else {
        return [];
      }
    },
    isEditMode(): boolean {
      return this.mode == "edit";
    },
    application(): ApplicationEntity {
      return this.$store.getters["application/get"]({ id: this.applicationId });
    },
  },
});
</script>

<style type="text/css" scoped>
#application-summary {
  background-color: #292f32;
}

.button-standard {
  background-color: #50928b;
}

.button-standard:hover {
  background-color: #42a69b;
}

.button-save {
  background-color: #2da06f;
}
.button-save:hover {
  background-color: #32b27b;
}

.button-abort {
  background-color: #a94d50;
}

.button-abort:hover {
}

.card-section {
  background-color: #242a2c;
  border-color: #384145;
}
</style>
