<template>
  <div id="application-details" class="flex flex-col flex-no-wrap">
    <div id="application-summary" class="flex-none w-full h-40">
      {{ systemName }}

      <div class="flex flex-row-reverse pr-8 pb-4">

        <button
          class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
          @click="execute()"
          type="button"
        >
          execute
        </button>

        <div>
          <Dropdown class="mr-8" :default="options[0]" :options="options" />
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
import Dropdown from "@/components/ui/Dropdown";
import { mapState, mapActions } from "vuex";
import { registry } from "si-registry";

export default {
  name: "ApplicationDetails",
  components: {
    Editor,
    Dropdown,
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
      options: ["orange", "blue", "red"],
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
