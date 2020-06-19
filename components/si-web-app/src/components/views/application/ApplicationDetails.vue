<template>
  <div id="application-details" class="flex flex-col flex-no-wrap">
    
    <div id="application-summary" class="flex flex-col w-full h-40">
      
      <StatusBar class=""/>

      <div class="flex flex-row">

        <div class="flex flex-col w-2/3">
          <div class="text-white">
            Applications/{{ systemName }}
          </div>

          <div class="flex mx-4 my-4 w-full justify-start">
            <ActivityVisualization class="mx-2"/>
            <ServicesVisualization class="mx-2"/>
            <SystemsVisualization class="mx-2"/>
          </div>
        </div>

        <div class="flex flex-col w-1/3 justify-end">

          <div class="flex text-sm text-gray-400">
            <div class="block font-light">
              ChangeSet:
            </div>
            <div class="font-normal">
              <Dropdown
                class="ml-1"
                :optionDefault="options[0]"
                :optionList="options"
                optionListOrientation="left"
              />
            </div>
          </div>

        <div class="flex flex-row-reverse pr-4 mt-4">

          <button
            class="bg-teal-700 px-2 py-1 text-white text-sm hover:bg-teal-600"
            @click="execute()"
            type="button"
          >
            execute
          </button>

          <button
            class="bg-teal-700 mr-2 px-2 py-1 text-white text-sm hover:bg-teal-600"
            @click="execute()"
            type="button"
          >
            edit
          </button>

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
import Editor from "@/components/views/editor/Editor.vue";
import StatusBar from "@/components/common/StatusBar.vue"
import ServicesVisualization from "@/components/visualization/ServicesVisualization.vue"
import SystemsVisualization from "@/components/visualization/SystemsVisualization.vue"
import ActivityVisualization from "@/components/visualization/ActivityVisualization.vue"
import Dropdown from "@/components/ui/Dropdown";

import { mapState, mapActions } from "vuex";
import { registry } from "si-registry";

export default {
  name: "ApplicationDetails",
  components: {
    Editor,
    StatusBar,
    ServicesVisualization,
    SystemsVisualization,
    ActivityVisualization,
    Dropdown
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
      options: ["alex/my-glorious-changeset", "blue", "red"],
      app: {
        id: this.applicationId
      }
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
  background-color: #2a2f32;
}
</style>
