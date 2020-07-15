<template>
  <div id="system-details" class="flex flex-col flex-no-wrap">
    <div id="system-summary" class="flex-none h-40">
      <div>
        <span class="text-white">
          Saving: {{ isSaving ? "true" : "false" }}
        </span>
        <span v-if="editSaveError" class="text-white">
          Save Error: {{ editSaveError.message }}
        </span>
      </div>
      <span class="text-white">
        Mode: {{ mode }} System: {{ systemName }} Change Set Status:
        {{ changeSet.status }} Change Sets:
        <select
          label="Change Sets"
          aria-label="Change Sets"
          class="bg-gray-800 border text-gray-400 text-sm px-4 leading-tight focus:outline-none"
          v-model="selectedChangeSetId"
        >
          <option
            v-for="changeSet in changeSets"
            :key="changeSet.id"
            :value="changeSet.id"
          >
            {{ changeSet.name }} ({{ changeSet.status }})
          </option>
        </select>
      </span>

      <div class="flex flex-row-reverse pr-8 pb-4">
        <button
          class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
          @click="createChangeSet()"
          type="button"
        >
          new changeSet
        </button>

        <button
          class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
          @click="execute()"
          type="button"
        >
          execute
        </button>
        <button
          class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
          @click="deleteNode()"
          type="button"
        >
          delete
        </button>

        <button
          class="bg-teal-700 px-4 py-2 text-white hover:bg-teal-600"
          @click="modeSwitch()"
          type="button"
        >
          <template v-if="mode == 'view'">
            edit
          </template>
          <template v-else>
            view
          </template>
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
import { mapState, mapActions } from "vuex";
import { registry } from "si-registry";

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
    createChangeSet() {
      this.$store.dispatch("changeSet/createDefault");
    },
    execute() {
      this.$store.dispatch("changeSet/execute");
    },
    deleteNode() {
      this.$store.dispatch("node/delete");
    },
    modeSwitch() {
      this.$store.dispatch("editor/modeSwitch");
    },
  },
  computed: {
    selectedChangeSetId: {
      get() {
        return this.changeSet.id;
      },
      async set(value) {
        await this.$store.commit("changeSet/setCurrentById", value);
      },
    },
    ...mapState({
      changeSet: state => state.changeSet.current || {},
      changeSets: state => state.changeSet.changeSets,
      mode: state => state.editor.mode,
      isSaving: state => state.editor.isSaving,
      editSaveError: state => state.editor.editSaveError,
    }),
  },
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
