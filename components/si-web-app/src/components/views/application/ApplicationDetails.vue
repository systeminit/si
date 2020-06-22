<template>
  <div id="application-details" class="flex flex-col flex-no-wrap">
    <div id="application-summary" class="flex flex-col w-full h-40">
      <StatusBar class="" />

      <div class="flex flex-col">
        <div class="flex flex-row mt-3 mx-3">
          <div class="text-gray-300 font-normal">
            Applications/{{ systemName }}
          </div>

          <div class="flex w-full flex-row-reverse">
            <button
              class="mx-1 px-1 h-7 w-auto text-white text-sm button-standard"
              @click="execute()"
              type="button"
            >
              <div class="flex">
                <play-icon size="1.25x" class="self-center text-gray-200" />
                <div class="ml-1 font-normal text-gray-100">execute</div>
              </div>
            </button>

            <div v-if="isEditMode === false" class="flex">
              <button
                class="mx-1 px-2 h-7 w-auto text-white text-sm button-standard"
                @click="toggleEditMode(true)"
                type="button"
              >
                <div class="flex">
                  <edit-icon size="1.25x" class="self-center text-gray-200" />
                  <div class="ml-1 font-normal text-gray-100">edit</div>
                </div>
              </button>
            </div>

            <div v-else-if="isEditMode === true" class="flex">
              <button
                class="mx-1 px-2 h-7 w-auto text-white text-sm button-save"
                @click="toggleEditMode(false)"
                type="button"
              >
                <div class="flex">
                  <save-icon size="1.25x" class="self-center text-gray-200" />
                  <div class="ml-1 font-normal text-gray-100">save</div>
                </div>
              </button>

              <button
                class="mx-1 px-2 h-7 w-auto text-white text-sm button-abort"
                @click="toggleEditMode(false)"
                type="button"
              >
                <div class="flex">
                  <trash-icon size="1.25x" class="self-center text-gray-200" />
                  <div class="ml-1 font-normal text-gray-100">abort</div>
                </div>
              </button>
            </div>
          </div>
        </div>

        <div class="flex mx-4 my-4 justify-start">
          <div class="mx-2 w-40 border card-section">
            <ActivityVisualization class="mx-2 my-1" />
          </div>

          <div class="mx-2 w-3/12 border card-section">
            <ServicesVisualization class="mx-2 my-1" />
          </div>

          <div class="mx-2 w-3/12 border card-section">
            <div class="mx-2 my-1 text-sm font-bold text-gray-400">
              <div class="flex">
                <div>systems:</div>
                <div class="ml-1 font-normal">2</div>
              </div>

              <div class="flex mt-1">
                <div class="ml-1 font-light">system:</div>

                <div class="ml-1 w-full">
                  <Dropdown
                    class="w-auto"
                    :optionDefault="systems[0]"
                    :optionList="systems"
                    menuStyle="standard"
                  />
                </div>
              </div>
            </div>
          </div>

          <div class="mx-2 w-4/12 border card-section">
            <div class="mx-2 my-1 text-sm font-bold text-gray-400">
              <div class="flex">
                <div>Changes:</div>
                <div class="ml-1 font-normal">3</div>
                <alert-circle-icon
                  size="1x"
                  class="ml-1 self-center text-orange-600"
                />
              </div>

              <div class="flex mt-1">
                <div class="ml-1 font-light">changeset:</div>

                <div class="ml-1 w-full">
                  <Dropdown
                    class="w-auto"
                    :optionDefault="changesets[0]"
                    :optionList="changesets"
                    menuStyle="standard"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div id="editor" class="flex h-full w-full overflow-hidden">
      <Editor />
    </div>
  </div>
</template>

<script>
import { mapState, mapActions } from "vuex";
import { registry } from "si-registry";

import Editor from "@/components/views/editor/Editor.vue";
import StatusBar from "@/components/common/StatusBar.vue";
import ServicesVisualization from "@/components/visualization/ServicesVisualization.vue";
import ActivityVisualization from "@/components/visualization/ActivityVisualization.vue";
import Dropdown from "@/components/ui/Dropdown";

import {
  PlayIcon,
  EditIcon,
  AlertCircleIcon,
  TrashIcon,
  SaveIcon,
} from "vue-feather-icons";

export default {
  name: "ApplicationDetails",
  components: {
    Editor,
    StatusBar,
    ServicesVisualization,
    ActivityVisualization,
    Dropdown,
    PlayIcon,
    EditIcon,
    AlertCircleIcon,
    TrashIcon,
    SaveIcon,
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
  data: function() {
    return {
      systemName: "demo",
      systems: ["dev", "production"],
      changesets: ["a changeset", "another changeset", "the changeset"],
      app: {
        id: this.applicationId,
      },
      isEditMode: false,
    };
  },
  methods: {
    execute() {
      try {
        let mutation = registry
          .get("kubernetesDeploymentEntity")
          .graphql.mutation({ methodName: "apply" });
        let variables = registry
          .get("kubernetesDeploymentEntity")
          .graphql.variablesObject({ methodName: "apply" });

        variables.id = this.selectedNode.id;

        this.$apollo.mutate({
          mutation,
          variables,
        });
      } catch (error) {
        console.log("error", { error });
      }
    },
    toggleEditMode(value) {
      this.isEditMode = value;
      console.log(value);
    },
  },
  computed: {
    ...mapState({
      selectedNode: state => state.editor.selectedNode,
    }),
  },
  // watch: {
  //   selectedNode (newState, previousState) {
  //     // console.log("new state:", newState)
  //   }
  // },
};
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
