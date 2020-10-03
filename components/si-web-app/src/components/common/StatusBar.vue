<template>
  <div class="flex items-center w-full h-6 status-bar">
    <div class="ml-1 text-xs text-gray-400">
      {{ application }}
    </div>
    <ChevronsRightIcon size="1.0x" class="text-gray-400" />
    <div class="text-xs text-gray-400">
      {{ system }}
    </div>
    <ChevronsRightIcon size="1.0x" class="text-gray-400" />
    <div class="text-xs text-gray-400">{{ node }} [{{ nodeType }}]</div>
    <div class="flex justify-end flex-grow mr-1">
      <div class="w-6 mr-1 text-xs" v-bind:class="modeClasses">
        {{ mode }}
      </div>

      <GitBranchIcon size="1.0x" class="mr-1 text-gray-400" />
      <div class="mr-1 text-xs text-gray-400">
        {{ changeSet }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { ChevronsRightIcon, GitBranchIcon } from "vue-feather-icons";
import { mapState } from "vuex";
import { camelCase } from "change-case";

export default Vue.extend({
  name: "StatusBar",
  components: {
    ChevronsRightIcon,
    GitBranchIcon,
  },
  computed: {
    modeClasses(): Record<string, boolean> {
      const result: Record<string, boolean> = {};
      if (this.mode == "edit") {
        result["mode-edit"] = true;
      } else {
        result["mode-view"] = true;
      }
      return result;
    },
    nodeType(): string {
      return this.editObject?.objectType || "none";
    },
    node(): string {
      return this.editObject?.name || "none";
    },
    ...mapState({
      mode: (state: any): string => state.editor.mode,
      application: (state: any): string =>
        state.editor.application?.name || "none",
      system: (state: any): string => state.editor.system?.name || "none",
      changeSet: (state: any): string => state.editor.changeSet?.name || "none",
      changeSetId: (state: any): string =>
        state.changeSet.changeSet?.id || "none",
      editObject: (state: any): any => state.editor.editObject,
    }),
  },
});
</script>

<style>
.status-bar {
  background-color: #1f2324;
}

.mode-edit {
  color: #d9a35e;
}
.mode-view {
  color: #3091c1;
}
</style>
