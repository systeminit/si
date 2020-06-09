<template>
  <div id="system-details" class="flex flex-col flex-no-wrap">
    

    <div id="system-summary" class="flex-none h-40">
      {{ systemName }}

      <div class="flex flex-row-reverse pr-8 pb-4">
        <button class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600" @click="execute()" type="button">
          execute
        </button>
      </div>

    </div>


    <div id="system-editor" class="flex h-full w-full overflow-hidden">
      <Editor />
    </div>
  </div>
</template>

<script>
import Editor from "@/components/views/editor/Editor.vue";
import { mapState, mapActions } from 'vuex'

export default {
  name: "SystemDetails",
  components: {
    Editor,
  },
  data: function() {
    return {
      systemName: "demo",
    };
  },
  methods: {
    execute() {
      console.log("executing", this.selectedNode)
    }
  },
  computed: {
    ...mapState({
      selectedNode: state => state.editor.selectedNode
    }),
  },
  watch: {
    selectedNode (newState, previousState) {
      console.log("new state:", newState)
    }
  }

};
</script>

<style type="text/css" scoped>
#system-summary {
  background-color: #2a2f32;
}

#system-editor {
  background-color: #000000;
}
</style>
